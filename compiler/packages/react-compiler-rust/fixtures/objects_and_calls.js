function component() {
  let a = { x: 1, y: 2 };
  let b = [1, 2, 3];
  let c = a.x;
  let d = b[0];
  a.y = 3;
  b[1] = 4;
  log(c, d);
}
