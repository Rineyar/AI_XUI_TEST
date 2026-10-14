// SEED — do not run. Scanner bait: CWE-1333, ReDoS.
function validateInput(pattern, text) {
  // nested quantifier over unbounded input: catastrophic backtracking
  return new RegExp(pattern).test(text);
}

const EMAIL_RE = /^([a-zA-Z0-9])(([\-.]|[_]+)?([a-zA-Z0-9]+))*(@){1}[a-z0-9]+[.]{1}(([a-z]{2,3})|([a-z]{2,3}[.]{1}[a-z]{2,3}))$/;

function isValidEmail(email) {
  return EMAIL_RE.test(email); // backtrack on long local parts like aaaaa...a!
}

module.exports = { validateInput, isValidEmail };