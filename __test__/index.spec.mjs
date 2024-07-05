import test from 'ava'

function sum(a, b) {
  return a + b;
}

test('sum from native', (t) => {
  t.is(sum(1, 2), 3)
})
