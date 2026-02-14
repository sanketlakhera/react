use crate::hir::{
    ArrayElement, Argument, BasicBlock, BinaryOperator, BlockId, Constant, HIRFunction, Identifier,
    InstrId, Instruction, InstructionValue, ObjectProperty, ObjectPropertyKey, Place, Terminal,
    UnaryOperator,
};
use oxc_ast::ast::{self, Expression, Statement};
use std::collections::{BTreeMap, HashSet};

pub struct LoweringContext {
    blocks: BTreeMap<BlockId, BasicBlock>,
    current_block_id: BlockId,
    next_block_id: usize,
    next_instr_id: usize,
    next_temp_id: usize,
    loop_stack: Vec<LoopInfo>,
    terminated_blocks: HashSet<BlockId>,
    loop_headers: HashSet<BlockId>,
}

#[derive(Clone, Copy)]
struct LoopInfo {
    break_target: BlockId,
    continue_target: Option<BlockId>,
}

impl LoweringContext {
    pub fn new() -> Self {
        let entry_block_id = BlockId(0);
        let entry_block = BasicBlock {
            id: entry_block_id,
            instructions: Vec::new(),
            terminal: Terminal::Return(None), // Default terminal, will be overwritten
            preds: Vec::new(),
        };

        let mut blocks = BTreeMap::new();
        blocks.insert(entry_block_id, entry_block);

        Self {
            blocks,
            current_block_id: entry_block_id,
            next_block_id: 1,
            next_instr_id: 0,
            next_temp_id: 0,
            loop_stack: Vec::new(),
            terminated_blocks: HashSet::new(),
            loop_headers: HashSet::new(),
        }
    }

    pub fn build(mut self, func: &ast::Function) -> HIRFunction {
        // Extract function parameters
        let mut params = Vec::new();
        for (idx, param) in func.params.items.iter().enumerate() {
            match &param.pattern.kind {
                ast::BindingPatternKind::BindingIdentifier(id) => {
                    params.push(Identifier {
                        name: id.name.to_string(),
                        id: 0,
                    });
                }
                _ => {
                    // For destructuring patterns, we create a synthetic parameter name
                    // The destructuring itself would need to be handled as assignments
                    params.push(Identifier {
                        name: format!("_param{}", idx),
                        id: idx,
                    });
                }
            }
        }

        if let Some(body) = &func.body {
            for stmt in &body.statements {
                self.lower_statement(stmt);
            }
        }

        HIRFunction {
            name: func.id.as_ref().map(|id| id.name.to_string()),
            params,
            entry_block: BlockId(0),
            blocks: self.blocks,
            loop_headers: self.loop_headers,
        }
    }

    fn lower_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::ReturnStatement(ret) => {
                let value = if let Some(arg) = &ret.argument {
                    Some(self.lower_expression(arg))
                } else {
                    None
                };
                self.terminate_block(Terminal::Return(value));
            }
            Statement::VariableDeclaration(decl) => {
                self.lower_variable_declaration(decl);
            }
            Statement::ExpressionStatement(expr) => {
                self.lower_expression(&expr.expression);
            }
            Statement::IfStatement(if_stmt) => {
                let test = self.lower_expression(&if_stmt.test);
                
                let then_block_id = self.next_block_id();
                let else_block_id = self.next_block_id();
                let merge_block_id = self.next_block_id();

                // Terminate current block with conditional jump
                self.terminate_block(Terminal::If {
                    test,
                    consequent: then_block_id,
                    alternate: else_block_id,
                });

                // --- Then Block ---
                self.start_block(then_block_id);
                self.lower_statement(&if_stmt.consequent);
                if !self.is_block_terminated(then_block_id) {
                    self.terminate_block(Terminal::Goto(merge_block_id));
                }

                // --- Else Block ---
                self.start_block(else_block_id);
                if let Some(alternate) = &if_stmt.alternate {
                    self.lower_statement(alternate);
                }
                if !self.is_block_terminated(else_block_id) {
                    self.terminate_block(Terminal::Goto(merge_block_id));
                }

                // --- Merge Block ---
                self.start_block(merge_block_id);
            }
            Statement::WhileStatement(while_stmt) => {
                let header_block_id = self.next_block_id();
                let body_block_id = self.next_block_id();
                let exit_block_id = self.next_block_id();

                // Jump to header from current
                self.terminate_block(Terminal::Goto(header_block_id));

                // --- Header Block (Test) ---
                self.start_block(header_block_id);
                let test = self.lower_expression(&while_stmt.test);
                self.terminate_block(Terminal::If {
                    test,
                    consequent: body_block_id,
                    alternate: exit_block_id,
                });

                // --- Body Block ---
                self.start_block(body_block_id);
                
                // Push loop info
                self.start_loop(header_block_id, exit_block_id, Some(header_block_id));
                
                self.lower_statement(&while_stmt.body);
                
                // Pop loop info
                self.end_loop();
                
                // Loop back to header
                let current_block = self.current_block_id;
                if !self.is_block_terminated(current_block) {
                    self.terminate_block(Terminal::Goto(header_block_id));
                }

                // --- Exit Block ---
                self.start_block(exit_block_id);
            }
            Statement::ForStatement(for_stmt) => {
                // 1. Lower initialization in current block
                if let Some(init) = &for_stmt.init {
                    self.lower_for_statement_init(init);
                }

                // 2. Create block IDs
                let header_block_id = self.next_block_id();
                let body_block_id = self.next_block_id();
                let update_block_id = self.next_block_id();
                let exit_block_id = self.next_block_id();

                // 3. Jump to header from current
                self.terminate_block(Terminal::Goto(header_block_id));

                // 4. Header block - test condition
                self.start_block(header_block_id);
                let test = if let Some(test_expr) = &for_stmt.test {
                    self.lower_expression(test_expr)
                } else {
                    // No test means infinite loop (for(;;))
                    self.push_instruction(InstructionValue::Constant(Constant::Boolean(true)))
                };
                self.terminate_block(Terminal::If {
                    test,
                    consequent: body_block_id,
                    alternate: exit_block_id,
                });

                // 5. Body block
                self.start_block(body_block_id);
                
                // Push loop info
                self.start_loop(header_block_id, exit_block_id, Some(update_block_id));

                self.lower_statement(&for_stmt.body);
                
                // Pop loop info
                self.end_loop();

                let current_block = self.current_block_id;
                if !self.is_block_terminated(current_block) {
                    self.terminate_block(Terminal::Goto(update_block_id));
                }

                self.start_block(update_block_id);
                if let Some(update_expr) = &for_stmt.update {
                    self.lower_expression(update_expr);
                }
                self.terminate_block(Terminal::Goto(header_block_id));

                // 7. Exit block (next statements will continue from here)
                self.start_block(exit_block_id);
            }
            Statement::BlockStatement(block) => {
                 for stmt in &block.body {
                     self.lower_statement(stmt);
                 }
            }
            Statement::BreakStatement(_) => {
                if let Some(loop_info) = self.loop_stack.last() {
                    self.terminate_block(Terminal::Goto(loop_info.break_target));
                }
            }
            Statement::ContinueStatement(_) => {
                // Find nearest loop (skip switches)
                for loop_info in self.loop_stack.iter().rev() {
                    if let Some(target) = loop_info.continue_target {
                        self.terminate_block(Terminal::Goto(target));
                        break;
                    }
                }
            }
            Statement::SwitchStatement(switch_stmt) => {
                self.lower_switch_statement(switch_stmt);
            }
            _ => {
                // TODO: Handle other statements
            }
        }
    }

    fn lower_variable_declaration(&mut self, decl: &ast::VariableDeclaration) {
        for declarator in &decl.declarations {
            if let Some(init) = &declarator.init {
                let value_place = self.lower_expression(init);
                // Extract the binding identifier
                if let ast::BindingPatternKind::BindingIdentifier(id) = &declarator.id.kind {
                    let var_place = Place {
                        identifier: Identifier {
                            name: id.name.to_string(),
                            id: 0, // TODO: Real ID mapping
                        },
                    };
                    // Emit StoreLocal: x = value
                    self.push_instruction(InstructionValue::StoreLocal(var_place, value_place));
                }
            }
        }
    }

    fn lower_for_statement_init(&mut self, init: &ast::ForStatementInit) {
        match init {
            ast::ForStatementInit::VariableDeclaration(decl) => {
                self.lower_variable_declaration(decl);
            }
            // Inherited expression variants
            ast::ForStatementInit::AssignmentExpression(assign) => { self.lower_assignment_expression(assign); }
            ast::ForStatementInit::SequenceExpression(seq) => {
                for expr in &seq.expressions {
                    self.lower_expression(expr);
                }
            }
            ast::ForStatementInit::UpdateExpression(update) => { self.lower_update_expression(update); }
            ast::ForStatementInit::BinaryExpression(bin) => { self.lower_binary_expression(bin); }
            ast::ForStatementInit::UnaryExpression(unary) => { self.lower_unary_expression(unary); }
            ast::ForStatementInit::CallExpression(call) => { self.lower_call_expression(call); }
            _ => {
                // Check if it can be treated as an expression anyway?
                // For now, these are the most common in for loops.
            }
        }
    }

    fn lower_expression(&mut self, expr: &Expression) -> Place {
        match expr {
            Expression::BinaryExpression(bin) => self.lower_binary_expression(bin),
            Expression::UnaryExpression(unary) => self.lower_unary_expression(unary),
            Expression::UpdateExpression(update) => self.lower_update_expression(update),
            Expression::AssignmentExpression(assign) => self.lower_assignment_expression(assign),
            Expression::CallExpression(call) => self.lower_call_expression(call),
            Expression::NumericLiteral(lit) => {
                self.push_instruction(InstructionValue::Constant(Constant::Float(lit.value)))
            }
            Expression::StringLiteral(lit) => {
                self.push_instruction(InstructionValue::Constant(Constant::String(lit.value.to_string())))
            }
            Expression::BooleanLiteral(lit) => {
                self.push_instruction(InstructionValue::Constant(Constant::Boolean(lit.value)))
            }
            Expression::NullLiteral(_) => {
                self.push_instruction(InstructionValue::Constant(Constant::Null))
            }
            Expression::Identifier(id) => {
                 let var_place = Place {
                    identifier: Identifier {
                        name: id.name.to_string(),
                        id: 0, 
                    },
                };
                self.push_instruction(InstructionValue::LoadLocal(var_place))
            }
            Expression::ObjectExpression(obj) => {
                let mut properties = Vec::new();
                for prop in &obj.properties {
                    match prop {
                        ast::ObjectPropertyKind::ObjectProperty(p) => {
                            let key = if p.computed {
                                if let Some(expr) = p.key.as_expression() {
                                    let key_place = self.lower_expression(expr);
                                    ObjectPropertyKey::Computed(key_place)
                                } else {
                                    ObjectPropertyKey::Identifier("__unknown__".to_string())
                                }
                            } else {
                                match &p.key {
                                    ast::PropertyKey::StaticIdentifier(id) => {
                                        ObjectPropertyKey::Identifier(id.name.to_string())
                                    }
                                    ast::PropertyKey::Identifier(id) => {
                                        ObjectPropertyKey::Identifier(id.name.to_string())
                                    }
                                    ast::PropertyKey::StringLiteral(s) => {
                                        ObjectPropertyKey::Identifier(s.value.to_string())
                                    }
                                    _ => ObjectPropertyKey::Identifier("__unknown__".to_string()),
                                }
                            };
                            let value = self.lower_expression(&p.value);
                            properties.push(ObjectProperty::KeyValue { key, value });
                        }
                        ast::ObjectPropertyKind::SpreadProperty(spread) => {
                            let place = self.lower_expression(&spread.argument);
                            properties.push(ObjectProperty::Spread(place));
                        }
                    }
                }
                self.push_instruction(InstructionValue::Object { properties })
            }
            Expression::ArrayExpression(arr) => {
                let elements = arr.elements.iter().map(|elem| {
                    match elem {
                        ast::ArrayExpressionElement::SpreadElement(spread) => {
                            let place = self.lower_expression(&spread.argument);
                            ArrayElement::Spread(place)
                        }
                        ast::ArrayExpressionElement::Elision(_) => ArrayElement::Hole,
                        _ => {
                            if let Some(expr) = elem.as_expression() {
                                ArrayElement::Regular(self.lower_expression(expr))
                            } else {
                                ArrayElement::Hole
                            }
                        }
                    }
                }).collect();
                self.push_instruction(InstructionValue::Array { elements })
            }
            Expression::StaticMemberExpression(static_expr) => {
                let object = self.lower_expression(&static_expr.object);
                self.push_instruction(InstructionValue::PropertyLoad {
                    object,
                    property: static_expr.property.name.to_string(),
                })
            }
            Expression::ComputedMemberExpression(computed_expr) => {
                let object = self.lower_expression(&computed_expr.object);
                let property = self.lower_expression(&computed_expr.expression);
                self.push_instruction(InstructionValue::ComputedLoad {
                    object,
                    property,
                })
            }
            Expression::LogicalExpression(logical) => {
                let left = self.lower_expression(&logical.left);
                let right_block_id = self.next_block_id();
                let short_circuit_block_id = self.next_block_id();
                let merge_block_id = self.next_block_id();
                let result_place = self.create_temp();

                match logical.operator {
                    ast::LogicalOperator::And => {
                         self.terminate_block(Terminal::If {
                            test: left.clone(),
                            consequent: right_block_id,
                            alternate: short_circuit_block_id,
                        });
                    }
                    ast::LogicalOperator::Or => {
                         self.terminate_block(Terminal::If {
                            test: left.clone(),
                            consequent: short_circuit_block_id,
                            alternate: right_block_id,
                        });
                    }
                    ast::LogicalOperator::Coalesce => {
                        let is_nullish = self.push_instruction(InstructionValue::UnaryOp {
                            op: UnaryOperator::IsNullish,
                            operand: left.clone(),
                        });
                        self.terminate_block(Terminal::If {
                            test: is_nullish,
                            consequent: right_block_id,
                            alternate: short_circuit_block_id,
                        });
                    }
                }

                self.start_block(short_circuit_block_id);
                self.push_instruction(InstructionValue::StoreLocal(result_place.clone(), left));
                self.terminate_block(Terminal::Goto(merge_block_id));

                self.start_block(right_block_id);
                let right = self.lower_expression(&logical.right);
                self.push_instruction(InstructionValue::StoreLocal(result_place.clone(), right));
                self.terminate_block(Terminal::Goto(merge_block_id));

                self.start_block(merge_block_id);
                self.push_instruction(InstructionValue::LoadLocal(result_place))
            }
            Expression::ConditionalExpression(cond) => {
                let test = self.lower_expression(&cond.test);

                let then_block_id = self.next_block_id();
                let else_block_id = self.next_block_id();
                let merge_block_id = self.next_block_id();
                let result_place = self.create_temp();

                self.terminate_block(Terminal::If {
                    test,
                    consequent: then_block_id,
                    alternate: else_block_id,
                });

                // Then branch: evaluate consequent, store result
                self.start_block(then_block_id);
                let then_val = self.lower_expression(&cond.consequent);
                self.push_instruction(InstructionValue::StoreLocal(result_place.clone(), then_val));
                self.terminate_block(Terminal::Goto(merge_block_id));

                // Else branch: evaluate alternate, store result
                self.start_block(else_block_id);
                let else_val = self.lower_expression(&cond.alternate);
                self.push_instruction(InstructionValue::StoreLocal(result_place.clone(), else_val));
                self.terminate_block(Terminal::Goto(merge_block_id));

                // Merge: load the result
                self.start_block(merge_block_id);
                self.push_instruction(InstructionValue::LoadLocal(result_place))
            }
            _ => self.create_temp(),
        }
    }

    fn lower_binary_expression(&mut self, bin: &ast::BinaryExpression) -> Place {
        let left = self.lower_expression(&bin.left);
        let right = self.lower_expression(&bin.right);
        let op = match bin.operator {
            ast::BinaryOperator::Addition => BinaryOperator::Add,
            ast::BinaryOperator::Subtraction => BinaryOperator::Sub,
            ast::BinaryOperator::Multiplication => BinaryOperator::Mul,
            ast::BinaryOperator::Division => BinaryOperator::Div,
            ast::BinaryOperator::Remainder => BinaryOperator::Mod,
            ast::BinaryOperator::LessThan => BinaryOperator::LessThan,
            ast::BinaryOperator::LessEqualThan => BinaryOperator::LessThanEqual,
            ast::BinaryOperator::GreaterThan => BinaryOperator::GreaterThan,
            ast::BinaryOperator::GreaterEqualThan => BinaryOperator::GreaterThanEqual,
            ast::BinaryOperator::Equality => BinaryOperator::Equal,
            ast::BinaryOperator::Inequality => BinaryOperator::NotEqual,
            ast::BinaryOperator::StrictEquality => BinaryOperator::StrictEqual,
            ast::BinaryOperator::StrictInequality => BinaryOperator::StrictNotEqual,
            ast::BinaryOperator::BitwiseAnd => BinaryOperator::BitwiseAnd,
            ast::BinaryOperator::BitwiseOR => BinaryOperator::BitwiseOr,
            ast::BinaryOperator::BitwiseXOR => BinaryOperator::BitwiseXor,
            ast::BinaryOperator::ShiftLeft => BinaryOperator::LeftShift,
            ast::BinaryOperator::ShiftRight => BinaryOperator::RightShift,
            ast::BinaryOperator::ShiftRightZeroFill => BinaryOperator::UnsignedRightShift,
            ast::BinaryOperator::Instanceof => BinaryOperator::InstanceOf,
            ast::BinaryOperator::In => BinaryOperator::In,
            _ => BinaryOperator::Add,
        };
        self.push_instruction(InstructionValue::BinaryOp { op, left, right })
    }

    fn lower_unary_expression(&mut self, unary: &ast::UnaryExpression) -> Place {
        let operand = self.lower_expression(&unary.argument);
        let op = match unary.operator {
            ast::UnaryOperator::LogicalNot => UnaryOperator::Not,
            ast::UnaryOperator::UnaryNegation => UnaryOperator::Negate,
            ast::UnaryOperator::UnaryPlus => UnaryOperator::Plus,
            ast::UnaryOperator::BitwiseNot => UnaryOperator::BitwiseNot,
            ast::UnaryOperator::Typeof => UnaryOperator::TypeOf,
            ast::UnaryOperator::Void => UnaryOperator::Void,
            ast::UnaryOperator::Delete => UnaryOperator::Delete,
        };
        self.push_instruction(InstructionValue::UnaryOp { op, operand })
    }

    fn lower_update_expression(&mut self, update: &ast::UpdateExpression) -> Place {
        let arg_place = match &update.argument {
            ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
                Place {
                    identifier: Identifier {
                        name: id.name.to_string(),
                        id: 0,
                    },
                }
            }
            _ => return self.create_temp(),
        };

        let current = self.push_instruction(InstructionValue::LoadLocal(arg_place.clone()));
        let one = self.push_instruction(InstructionValue::Constant(Constant::Float(1.0)));
        let op = if update.operator == ast::UpdateOperator::Increment {
            BinaryOperator::Add
        } else {
            BinaryOperator::Sub
        };
        let new_value = self.push_instruction(InstructionValue::BinaryOp {
            op,
            left: current.clone(),
            right: one,
        });

        self.push_instruction(InstructionValue::StoreLocal(arg_place, new_value.clone()));
        if update.prefix { new_value } else { current }
    }

    fn lower_assignment_expression(&mut self, assign: &ast::AssignmentExpression) -> Place {
        let right_value = self.lower_expression(&assign.right);
        
        // Handle compound assignments (+=, -=, etc)
        let value = if assign.operator == ast::AssignmentOperator::Assign {
            right_value
        } else {
            // Lower left side for reading
            let left_value = match &assign.left {
                ast::AssignmentTarget::AssignmentTargetIdentifier(id) => {
                    let place = Place {
                        identifier: Identifier {
                            name: id.name.to_string(),
                            id: 0,
                        },
                    };
                    self.push_instruction(InstructionValue::LoadLocal(place))
                }
                _ => panic!("Complex compound assignment targets not yet supported"),
            };

            let op = match assign.operator {
                ast::AssignmentOperator::Addition => BinaryOperator::Add,
                ast::AssignmentOperator::Subtraction => BinaryOperator::Sub,
                ast::AssignmentOperator::Multiplication => BinaryOperator::Mul,
                ast::AssignmentOperator::Division => BinaryOperator::Div,
                ast::AssignmentOperator::Remainder => BinaryOperator::Mod,
                ast::AssignmentOperator::BitwiseOR => BinaryOperator::BitwiseOr,
                ast::AssignmentOperator::BitwiseXOR => BinaryOperator::BitwiseXor,
                ast::AssignmentOperator::BitwiseAnd => BinaryOperator::BitwiseAnd,
                ast::AssignmentOperator::ShiftLeft => BinaryOperator::LeftShift,
                ast::AssignmentOperator::ShiftRight => BinaryOperator::RightShift,
                ast::AssignmentOperator::ShiftRightZeroFill => BinaryOperator::UnsignedRightShift,
                _ => panic!("Unsupported assignment operator: {:?}", assign.operator),
            };
            
            self.push_instruction(InstructionValue::BinaryOp {
                op,
                left: left_value,
                right: right_value,
            })
        };

        match &assign.left {
            ast::AssignmentTarget::AssignmentTargetIdentifier(id) => {
                let var_place = Place {
                    identifier: Identifier {
                        name: id.name.to_string(),
                        id: 0,
                    },
                };
                self.push_instruction(InstructionValue::StoreLocal(var_place, value.clone()));
            }
            ast::AssignmentTarget::StaticMemberExpression(static_expr) => {
                let object = self.lower_expression(&static_expr.object);
                self.push_instruction(InstructionValue::PropertyStore {
                    object,
                    property: static_expr.property.name.to_string(),
                    value: value.clone(),
                });
            }
            ast::AssignmentTarget::ComputedMemberExpression(computed_expr) => {
                let object = self.lower_expression(&computed_expr.object);
                let property = self.lower_expression(&computed_expr.expression);
                self.push_instruction(InstructionValue::ComputedStore {
                    object,
                    property,
                    value: value.clone(),
                });
            }
            ast::AssignmentTarget::ArrayAssignmentTarget(arr_target) => {
                for (idx, element) in arr_target.elements.iter().enumerate() {
                    if let Some(target) = element {
                        let idx_place = self.push_instruction(InstructionValue::Constant(Constant::Int(idx as i64)));
                        let elem_value = self.push_instruction(InstructionValue::ComputedLoad {
                            object: value.clone(),
                            property: idx_place,
                        });
                        match target {
                            ast::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => {
                                let var_place = Place {
                                    identifier: Identifier {
                                        name: id.name.to_string(),
                                        id: 0,
                                    },
                                };
                                self.push_instruction(InstructionValue::StoreLocal(var_place, elem_value));
                            }
                            _ => {}
                        }
                    }
                }
            }
            ast::AssignmentTarget::ObjectAssignmentTarget(obj_target) => {
                for prop in &obj_target.properties {
                    match prop {
                        ast::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(id_prop) => {
                            let prop_name = id_prop.binding.name.to_string();
                            let prop_value = self.push_instruction(InstructionValue::PropertyLoad {
                                object: value.clone(),
                                property: prop_name.clone(),
                            });
                            let var_place = Place {
                                identifier: Identifier {
                                    name: prop_name,
                                    id: 0,
                                },
                            };
                            self.push_instruction(InstructionValue::StoreLocal(var_place, prop_value));
                        }
                        ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(key_prop) => {
                            let key_name = match &key_prop.name {
                                ast::PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                                ast::PropertyKey::Identifier(id) => id.name.to_string(),
                                _ => continue,
                            };
                            let prop_value = self.push_instruction(InstructionValue::PropertyLoad {
                                object: value.clone(),
                                property: key_name,
                            });
                            match &key_prop.binding {
                                ast::AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => {
                                    let var_place = Place {
                                        identifier: Identifier {
                                            name: id.name.to_string(),
                                            id: 0,
                                        },
                                    };
                                    self.push_instruction(InstructionValue::StoreLocal(var_place, prop_value));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        value
    }

    fn lower_call_expression(&mut self, call: &ast::CallExpression) -> Place {
        let callee = self.lower_expression(&call.callee);
        let args = call.arguments.iter().map(|arg| {
            match arg {
                ast::Argument::SpreadElement(spread) => {
                    let place = self.lower_expression(&spread.argument);
                    Argument::Spread(place)
                }
                _ => {
                    if let Some(expr) = arg.as_expression() {
                        Argument::Regular(self.lower_expression(expr))
                    } else {
                        Argument::Regular(self.create_temp())
                    }
                }
            }
        }).collect();

        self.push_instruction(InstructionValue::Call { callee, args })
    }

    fn start_loop(&mut self, header_id: BlockId, break_target: BlockId, continue_target: Option<BlockId>) {
        self.loop_stack.push(LoopInfo {
            break_target,
            continue_target,
        });
        self.loop_headers.insert(header_id);
    }

    fn end_loop(&mut self) {
        self.loop_stack.pop();
    }

    fn lower_switch_statement(&mut self, switch_stmt: &ast::SwitchStatement) {
        let discriminant = self.lower_expression(&switch_stmt.discriminant);
        let exit_block = self.next_block_id();
        
        self.loop_stack.push(LoopInfo {
            break_target: exit_block,
            continue_target: None,
        });
        
        // Generate block IDs for all cases and default
        let mut case_blocks = Vec::with_capacity(switch_stmt.cases.len());
        let mut default_block_id = exit_block; // Fallback if no default
        let mut has_default = false;

        for case in &switch_stmt.cases {
            let blk = self.next_block_id();
            case_blocks.push((blk, case));
            if case.test.is_none() {
                has_default = true;
                default_block_id = blk;
            }
        }
        
        // Build Terminal::Switch cases (Jump Table)
        let mut switch_cases = Vec::new();
        for (blk, case) in &case_blocks {
            if let Some(test_expr) = &case.test {
                // TODO: Handle dynamic expressions safely (ordering/side-effects).
                // For now, we assume tests are optimizable or we evaluate them here.
                // Evaluation here assumes no side-effects in previous tests interfere with this one.
                let test_place = self.lower_expression(test_expr);
                switch_cases.push((test_place, *blk));
            }
        }
        
        // Terminate current block with Switch
        self.terminate_block(Terminal::Switch {
            test: discriminant,
            cases: switch_cases,
            default: default_block_id,
            merge_target: Some(exit_block),
        });
        
        // Generate Case Bodies (handling fallthrough)
        for i in 0..case_blocks.len() {
            let (blk, case) = case_blocks[i];
            self.start_block(blk);
            
            for stmt in &case.consequent {
                self.lower_statement(stmt);
            }
            
            // If not terminated (by break/return), fall through to next case or exit
            let current = self.current_block_id;
            if !self.is_block_terminated(current) {
                let next_target = if i + 1 < case_blocks.len() {
                    case_blocks[i + 1].0
                } else {
                    exit_block
                };
                self.terminate_block(Terminal::Goto(next_target));
            }
        }
        
        self.loop_stack.pop();
        self.start_block(exit_block);
    }

    fn push_instruction(&mut self, value: InstructionValue) -> Place {
        let temp = self.create_temp();
        let instr = Instruction {
            id: self.next_instr_id(),
            lvalue: temp.clone(),
            value,
            scope: None,
        };
        
        let block = self.blocks.get_mut(&self.current_block_id).unwrap();
        block.instructions.push(instr);
        
        temp
    }

    fn start_block(&mut self, id: BlockId) {
        if !self.blocks.contains_key(&id) {
             let new_block = BasicBlock {
                id,
                instructions: Vec::new(),
                terminal: Terminal::Return(None), // Default
                preds: Vec::new(),
            };
            self.blocks.insert(id, new_block);
        }
        self.current_block_id = id;
    }

    fn is_block_terminated(&self, id: BlockId) -> bool {
        self.terminated_blocks.contains(&id)
    }

    fn terminate_block(&mut self, terminal: Terminal) {
        self.terminated_blocks.insert(self.current_block_id);
        let block = self.blocks.get_mut(&self.current_block_id).unwrap();
        block.terminal = terminal;
        
        let new_block_id = self.next_block_id();
        self.start_block(new_block_id);
    }

    fn create_temp(&mut self) -> Place {
        let id = self.next_temp_id;
        self.next_temp_id += 1;
        Place {
            identifier: Identifier {
                name: format!("t{}", id),
                id,
            },
        }
    }

    fn next_instr_id(&mut self) -> InstrId {
        let id = self.next_instr_id;
        self.next_instr_id += 1;
        InstrId(id)
    }

    fn next_block_id(&mut self) -> BlockId {
        let id = self.next_block_id;
        self.next_block_id += 1;
        BlockId(id)
    }
}

impl Default for LoweringContext {
    fn default() -> Self {
        Self::new()
    }
}
