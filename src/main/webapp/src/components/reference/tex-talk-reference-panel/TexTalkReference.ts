import {
  TerminalDocItem,
  UnionDocItem,
  ref,
  literal,
  oneOrMoreTimes,
  zeroOrOneTime,
  zeroOrMoreTimes,
  group,
  DocItem,
} from '../Reference';

const expressionDocName = 'expression';
const commandLhsDocName = 'commandLhs';
const operatorCommandLhsDocName = 'operatorCommandLhs';

const numberDoc: TerminalDocItem = {
  name: 'number',
  form:
    group('-|+') +
    zeroOrOneTime() +
    '[0-9]+' +
    group(literal('.') + '[0-9]+') +
    zeroOrMoreTimes(),
  example: `
10
-10
0.123
-0.12
+0.12
+12
0
`,
};

const identifierDoc: TerminalDocItem = {
  name: 'identifier',
  form: '[a-zA-Z0-9]+',
  example: `
x
epsilon
f
someVar
`,
};

const identifierArgsDoc: TerminalDocItem = {
  name: 'identifierArgs',
  form:
    identifierDoc.name +
    group(literal(',') + ref(identifierDoc.name)) +
    zeroOrMoreTimes(),
  example: `
x, y
a, b, c
someVar
`,
};

const expressionArgsDoc: TerminalDocItem = {
  name: 'expressionArgs',
  form:
    expressionDocName +
    group(literal(',') + ref(expressionDocName)) +
    zeroOrMoreTimes(),
  example: `
\\continuous.function:on{A \\set.times/ B}:to{\\reals}
x + y
f(x) + g(x)
\\frac{\pi}{2}
`,
};

const functionMappingLhsDoc: TerminalDocItem = {
  name: 'functionMappingLhs',
  form:
    ref(identifierDoc.name) +
    literal('(') +
    ref(identifierArgsDoc.name) +
    literal(')'),
  example: `
f(x, y)
f(x)
Gamma(a, b, c)
SomeName(x, y)
`,
};

const functionMappingDoc: TerminalDocItem = {
  name: 'functionMapping',
  form:
    ref(identifierDoc.name) +
    literal('(') +
    ref(expressionArgsDoc.name) +
    literal(')'),
  example: `
f(x, y)
f(x + y, 10)
f(\\derivative[x]{x^2 + 1}, y)
`,
};

const sequenceMappingLhsDoc: TerminalDocItem = {
  name: 'sequenceMappingLhs',
  form:
    ref(identifierDoc.name) +
    literal('_') +
    literal('{') +
    ref(identifierArgsDoc.name) +
    literal('}'),
  example: `
x_{a, b, c},
f_{i}
`,
};

const sequenceMappingDoc: TerminalDocItem = {
  name: 'sequenceMapping',
  form:
    ref(identifierDoc.name) +
    literal('_') +
    literal('{') +
    ref(expressionArgsDoc.name) +
    literal('}'),
  example: `
x_{10, i + j}
x_{i}
x_{0}
`,
};

const functionLikeSequenceLhsDoc: TerminalDocItem = {
  name: 'functionLikeSequenceLhs',
  form:
    literal('{') +
    ref(identifierDoc.name) +
    literal('_') +
    literal('{') +
    ref(identifierArgsDoc.name) +
    literal('}') +
    literal('(') +
    ref(identifierArgsDoc.name) +
    literal(')') +
    literal('}') +
    literal('_') +
    literal('{') +
    ref(identifierArgsDoc.name) +
    literal('}'),
  example: `
{someName_{var1, var2}(arg1, arg2)}_{var1, var2}
{f_{i}(x)}_{i}
{f_{i,j}(x, y)}_{i,j}
`,
};

const nonFunctionLikeSequenceLhsDoc: TerminalDocItem = {
  name: 'nonFunctionLikeSequenceLhs',
  form:
    literal('{') +
    ref(identifierDoc.name) +
    literal('_') +
    literal('{') +
    ref(identifierArgsDoc.name) +
    literal('}') +
    literal('}') +
    literal('_') +
    literal('{') +
    ref(identifierArgsDoc.name) +
    literal('}'),
  example: `
{x_{i}}_{i}
{x_{i, j}}_{i, j}
`,
};

const sequenceLhsDoc: UnionDocItem = {
  name: 'sequenceLhs',
  allowed: [
    ref(functionLikeSequenceLhsDoc.name),
    ref(nonFunctionLikeSequenceLhsDoc.name),
  ],
};

const curlyGroupDoc: TerminalDocItem = {
  name: 'curlyGroup',
  form: literal('{') + ref(expressionDocName) + literal('}'),
  example: `
{x + y + \\frac{a}{b}}
{10}
`,
};

const parenGroupDoc: TerminalDocItem = {
  name: 'parenGroup',
  form: literal('(') + ref(expressionDocName) + literal(')'),
  example: `
(x + y + \\frac{a}{b})
(10)
`,
};

const operatorDoc: TerminalDocItem = {
  name: 'operator',
  form: '[~!@#$%^&*-+=|<>?/]+',
  example: `
+
++
+*
/
<=
`,
};

const tupleLhsDoc: TerminalDocItem = {
  name: 'tupleLhs',
  form: literal('(') + ref(identifierArgsDoc.name) + literal(')'),
  example: `
(X, Y, Z)
(x)
(a, b)
(name1, name2, name3)
`,
};

const tupleDoc: TerminalDocItem = {
  name: 'tuple',
  form: literal('(') + ref(expressionArgsDoc.name) + literal(')'),
  example: `
(10, x + y)
(\\f{a}, x + y)
`,
};

const lhsDocName = 'leftHandSide';
const lhsDoc: UnionDocItem = {
  name: lhsDocName,
  allowed: [
    ref(identifierDoc.name),
    ref(tupleLhsDoc.name),
    ref(functionMappingLhsDoc.name),
    ref(sequenceMappingLhsDoc.name),
    ref(commandLhsDocName),
    ref(operatorCommandLhsDocName),
    ref(sequenceLhsDoc.name),
  ],
};

const commandArgLhsDoc: UnionDocItem = {
  name: 'commandArgLhs',
  allowed: [
    ref(identifierDoc.name),
    ref(functionMappingLhsDoc.name),
    ref(sequenceMappingLhsDoc.name),
  ],
};

const commandArgsLhsDoc: TerminalDocItem = {
  name: 'commandArgs',
  form:
    ref(commandArgLhsDoc.name) +
    group(literal(',') + ref(commandArgLhsDoc.name)) +
    zeroOrMoreTimes(),
};

const commandPartParenGroupLhsDoc: TerminalDocItem = {
  name: 'commandPartParenGroupLhs',
  form: literal('(') + ref(commandArgsLhsDoc.name) + literal(')'),
};

const commandPartParenGroupDoc: TerminalDocItem = {
  name: 'commandPartParenGroup',
  form: literal('(') + ref(expressionArgsDoc.name) + literal(')'),
};

const commandPartCurlyGroupLhsDoc: TerminalDocItem = {
  name: 'commandPartCurlyGroupLhs',
  form: literal('{') + ref(commandArgsLhsDoc.name) + literal('}'),
};

const commandPartCurlyGroupDoc: TerminalDocItem = {
  name: 'commandPartCurlyGroup',
  form: literal('{') + ref(expressionArgsDoc.name) + literal('}'),
};

const commandPartNameDoc: TerminalDocItem = {
  name: 'commandPartName',
  form: '[a-zA-Z0-9~!@#$%^&*-+=|<>?/]' + oneOrMoreTimes(),
};

const commandPartNamedGroupLhsDoc: TerminalDocItem = {
  name: 'commandPartNamedGroupLhs',
  form:
    literal(':') +
    ref(commandPartNameDoc.name) +
    literal('{') +
    ref(commandArgsLhsDoc.name) +
    literal('}'),
};

const commandPartNamedGroupDoc: TerminalDocItem = {
  name: 'commandPartNamedGroup',
  form:
    literal(':') +
    ref(commandPartNameDoc.name) +
    literal('{') +
    ref(expressionArgsDoc.name) +
    literal('}'),
};

const commandPartSquareGroup: TerminalDocItem = {
  name: 'commandPartSquareGroup',
  form:
    literal('[') +
    ref(commandArgsLhsDoc.name) +
    literal(']') +
    group(
      literal('_') + literal('{') + ref(commandArgsLhsDoc.name) + literal('}')
    ) +
    zeroOrOneTime() +
    group(
      literal('^') + literal('{') + ref(commandArgsLhsDoc.name) + literal('}')
    ) +
    zeroOrOneTime(),
};

const commandPartLhsDoc: TerminalDocItem = {
  name: 'commandPartLhs',
  form:
    ref(commandPartNameDoc.name) +
    ref(commandPartSquareGroup.name) +
    zeroOrOneTime() +
    ref(commandPartCurlyGroupLhsDoc.name) +
    zeroOrMoreTimes() +
    ref(commandPartNamedGroupLhsDoc.name) +
    zeroOrMoreTimes(),
};

const commandPartDoc: TerminalDocItem = {
  name: 'commandPart',
  form:
    ref(commandPartNameDoc.name) +
    ref(commandPartSquareGroup.name) +
    zeroOrOneTime() +
    ref(commandPartCurlyGroupDoc.name) +
    zeroOrMoreTimes() +
    ref(commandPartNamedGroupDoc.name) +
    zeroOrMoreTimes(),
};

const commandLhsDoc: TerminalDocItem = {
  name: commandLhsDocName,
  form:
    literal('\\') +
    ref(commandPartLhsDoc.name) +
    oneOrMoreTimes() +
    ref(commandPartParenGroupLhsDoc.name) +
    zeroOrOneTime(),
  example: `
\\real.continuous.function:on{A}:to{B}
\\topological{G}.continuous.function:on{A}:to{B}
\\derivative[x]{f(x)}
\\limit[y]_{a}{f(y)}
\\finite.sum[k]_{a}^{b}{term(k)}
\\pi
\\someName
\\finite.group
`,
};

const commandDoc: TerminalDocItem = {
  name: 'command',
  form:
    literal('\\') +
    ref(commandPartDoc.name) +
    oneOrMoreTimes() +
    ref(commandPartParenGroupDoc.name) +
    zeroOrOneTime(),
  example: `
\\real.continuous.function:on{A \\set.times/ B}:to{\\reals}
\\topological{G := (X, O)}.continuous.function:on{A}:to{B}
\\derivative[x]{f(x) + g(x)}
\\limit[y]_{0}{y^2 + 2*y + 1}
\\finite.sum[k]_{0}^{10}{k^2 + 2}
\\pi
\\someName
`,
};

const operatorCommandLhsDoc: TerminalDocItem = {
  name: operatorCommandLhsDocName,
  form:
    ref(identifierDoc.name) +
    literal('\\') +
    ref(commandPartLhsDoc.name) +
    oneOrMoreTimes() +
    literal('/') +
    ref(identifierDoc.name),
  example: `
A \\set.times/ B
x \\leq{X}/ y
a \\otimes/ b
`,
};

const operatorCommandDoc: TerminalDocItem = {
  name: 'operatorCommand',
  form:
    ref(expressionDocName) +
    literal('\\') +
    ref(commandPartDoc.name) +
    oneOrMoreTimes() +
    literal('/') +
    ref(expressionDocName),
  example: `
A \\set.times/ B
x \\leq{X}/ 10
(A + B) \\otimes/ C
`,
};

const colonEqualsRhsDoc: UnionDocItem = {
  name: 'colonEqualsRhs',
  allowed: [ref(sequenceLhsDoc.name), ref(expressionDocName)],
};

const colonEqualsDoc: TerminalDocItem = {
  name: 'colonEquals',
  form: ref(lhsDoc.name) + literal(':=') + ref(colonEqualsRhsDoc.name),
  example: `
f(x) := x^2 + 1
X := (A \\otimes/ B, C, 0)
P := {p_{i}}_{i}`,
};

const isOrInLhsDoc: UnionDocItem = {
  name: 'isOrInLhs',
  allowed: [ref(lhsDoc.name), ref(colonEqualsDoc.name)],
};

const isDoc: TerminalDocItem = {
  name: 'is',
  form: ref(isOrInLhsDoc.name) + literal('is') + ref(commandDoc.name),
  example: `
x is \\real
f(x) is \\continuous.function
X := (a, b, c) is \\some.tuple
`,
};

const inDoc: TerminalDocItem = {
  name: 'in',
  form: ref(isOrInLhsDoc.name) + literal('in') + ref(expressionDocName),
  example: `
x in X
x in A \\set.times/ B
`,
};

const textDoc: TerminalDocItem = {
  name: 'text',
  form: literal('"') + '[^"]*' + literal('"'),
  example: `
"some text"
"let $x \\in X$.  Then there exists..."
`,
};

const expressionDoc: UnionDocItem = {
  name: expressionDocName,
  allowed: [
    ref(functionMappingDoc.name),
    ref(sequenceMappingDoc.name),
    ref(identifierDoc.name),
    ref(curlyGroupDoc.name),
    ref(tupleDoc.name),
    ref(parenGroupDoc.name),
    ref(isDoc.name),
    ref(inDoc.name),
    ref(operatorDoc.name),
    ref(colonEqualsDoc.name),
    ref(commandDoc.name),
    ref(operatorCommandDoc.name),
    ref(textDoc.name),
    ref(numberDoc.name),
  ],
  example: `
f(x) + g(x)
\\limit[i]_{\\infinity}{{a_{i}}_{i}}
2*x^2 + x + 1
x \\real.+/ y
{x + y} \\set.times// (A + B)
X := (a, b, c)
f(x) := \\sin(x)/x
`,
};

const docs = [
  identifierDoc,
  identifierArgsDoc,
  expressionArgsDoc,
  functionMappingLhsDoc,
  functionMappingDoc,
  sequenceMappingLhsDoc,
  sequenceMappingDoc,
  functionLikeSequenceLhsDoc,
  nonFunctionLikeSequenceLhsDoc,
  sequenceLhsDoc,
  curlyGroupDoc,
  parenGroupDoc,
  operatorDoc,
  tupleLhsDoc,
  tupleDoc,
  lhsDoc,
  commandArgLhsDoc,
  commandArgsLhsDoc,
  commandPartParenGroupLhsDoc,
  commandPartParenGroupDoc,
  commandPartCurlyGroupLhsDoc,
  commandPartCurlyGroupDoc,
  commandPartNameDoc,
  commandPartNamedGroupLhsDoc,
  commandPartNamedGroupDoc,
  commandPartSquareGroup,
  commandPartLhsDoc,
  commandPartDoc,
  commandLhsDoc,
  commandDoc,
  operatorCommandLhsDoc,
  operatorCommandDoc,
  colonEqualsRhsDoc,
  colonEqualsDoc,
  isOrInLhsDoc,
  isDoc,
  inDoc,
  textDoc,
  expressionDoc,
  numberDoc,
];

function buildNameToDocItems(): Map<string, DocItem> {
  const result: Map<string, DocItem> = new Map();
  for (const item of docs) {
    const refName = ref(item.name);
    if (result.has(refName)) {
      throw new Error(`Duplicate doc item found with name ${refName}`);
    }
    result.set(refName, item);
  }
  return result;
}

export function getDocItems(): DocItem[] {
  const nameToDocItem = buildNameToDocItems();
  const result: DocItem[] = [];
  getDocItemsImpl(ref(expressionDocName), result, nameToDocItem, new Set());
  return result;
}

function getDocItemsImpl(
  curRefName: string,
  result: DocItem[],
  nameToDocItem: Map<string, DocItem>,
  visited: Set<string>
) {
  if (visited.has(curRefName)) {
    return;
  }

  visited.add(curRefName);
  const curDocItem = nameToDocItem.get(curRefName)!;
  result.push(curDocItem);

  for (const name of getNeighborRefNames(curDocItem)) {
    getDocItemsImpl(name, result, nameToDocItem, visited);
  }
}

function getNeighborRefNames(docItem: any): string[] {
  if (docItem.form) {
    const result: string[] = [];
    for (const item of docs) {
      const refName = ref(item.name);
      if (docItem.form.indexOf(refName) >= 0) {
        result.push(refName);
      }
    }
    return result;
  } else if (docItem.allowed) {
    return docItem.allowed;
  } else {
    return [];
  }
}
