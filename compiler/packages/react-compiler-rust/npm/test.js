// Simple test for the native binding
const { compile, compileWithOptions, version } = require('./index');

console.log('React Compiler Rust - Test');
console.log('Version:', version());
console.log('');

// Test 1: Simple function
const result1 = compile(`
function greet(name) {
  const message = "Hello, " + name;
  return message;
}
`);

console.log('Test 1: Simple function');
console.log('Success:', result1.success);
if (result1.success) {
  console.log('Output:\n' + result1.code);
} else {
  console.log('Error:', result1.error);
}

console.log('');

// Test 2: React component (with jsx option)
const result2 = compileWithOptions(`
function Counter(props) {
  const doubled = props.count * 2;
  return <div>{doubled}</div>;
}
`, 'jsx');

console.log('Test 2: React JSX component');
console.log('Success:', result2.success);
if (result2.success) {
  console.log('Output:\n' + result2.code);
} else {
  console.log('Error:', result2.error);
}

console.log('');

// Test 3: TypeScript
const result3 = compileWithOptions(`
function add(a: number, b: number): number {
  const sum = a + b;
  return sum;
}
`, 'ts');

console.log('Test 3: TypeScript');
console.log('Success:', result3.success);
if (result3.success) {
  console.log('Output:\n' + result3.code);
} else {
  console.log('Error:', result3.error);
}
