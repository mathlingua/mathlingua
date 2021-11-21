import {
  DocItem,
  group,
  literal,
  ref,
  TerminalDocItem,
  UnionDocItem,
  zeroOrMoreTimes,
} from '../Reference';

const identifierDoc: TerminalDocItem = {
  name: 'identifier',
  form: '[a-zA-Z0-9~!@#$%^&*-+=|<>?/]+',
  example: `
X
+
0
++
10
`,
};

const identifierArgsDoc: TerminalDocItem = {
  name: 'identifierArgs',
  form:
    ref(identifierDoc.name) +
    group(literal(',') + ref(identifierDoc.name)) +
    zeroOrMoreTimes(),
  example: `
a, b
X, +, 0
`,
};

const tupleDoc: TerminalDocItem = {
  name: 'tuple',
  form: literal('(') + ref(identifierArgsDoc.name) + literal(')'),
  example: `
(a, b)
(X, +, 0)
`,
};

const functionMappingDoc: TerminalDocItem = {
  name: 'functionMapping',
  form:
    ref(identifierDoc.name) +
    literal('(') +
    ref(identifierArgsDoc.name) +
    literal(')'),
  example: `
f(x)
g(x, y, z)
`,
};

const functionLikeSequenceMappingDoc: TerminalDocItem = {
  name: 'functionLikeSequenceMapping',
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
{f_{i}(x)}_{i}
{f_{i,j}(x, y)}_{i, j}
`,
};

const nonFunctionLikeSequenceMappingDoc: TerminalDocItem = {
  name: 'nonFunctionLikeSequenceMapping',
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
{a_{i}}_{i}
{x_{i, j}}_{i, j}
`,
};

const sequenceMappingDoc: UnionDocItem = {
  name: 'sequenceMapping',
  allowed: [
    ref(functionLikeSequenceMappingDoc.name),
    ref(nonFunctionLikeSequenceMappingDoc.name),
  ],
};

const collectionDoc: TerminalDocItem = {
  name: 'collection',
  form: literal('{') + ref(identifierArgsDoc.name) + literal('}'),
  example: `
{a}
{x, y}
`,
};

const colonEqualsLhsDoc: UnionDocItem = {
  name: 'colonEqualsLhs',
  allowed: [
    ref(identifierDoc.name),
    ref(functionLikeSequenceMappingDoc.name) +
      ref(tupleDoc.name) +
      ref(collectionDoc.name),
  ],
};

const colonEqualsRhsDoc: UnionDocItem = {
  name: 'colonEqualsRhs',
  allowed: [ref(identifierDoc.name), ref(sequenceMappingDoc.name)],
};

const colonEqualsDoc: TerminalDocItem = {
  name: 'colonEquals',
  form:
    ref(colonEqualsLhsDoc.name) + literal(':=') + ref(colonEqualsRhsDoc.name),
  example: `
G := (X, +, 0)
X := {x}
X := {x_i}_i
`,
};

const targetDoc: UnionDocItem = {
  name: 'target',
  allowed: [
    ref(identifierDoc.name),
    ref(collectionDoc.name),
    ref(sequenceMappingDoc.name),
    ref(tupleDoc.name),
    ref(colonEqualsDoc.name),
  ],
};

export const docs = [
  identifierDoc,
  identifierArgsDoc,
  tupleDoc,
  functionMappingDoc,
  functionLikeSequenceMappingDoc,
  nonFunctionLikeSequenceMappingDoc,
  sequenceMappingDoc,
  collectionDoc,
  colonEqualsLhsDoc,
  colonEqualsRhsDoc,
  colonEqualsDoc,
  targetDoc,
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
  getDocItemsImpl(ref(targetDoc.name), result, nameToDocItem, new Set());
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
  if (curDocItem) {
    result.push(curDocItem);
  }

  for (const name of getNeighborRefNames(curDocItem)) {
    getDocItemsImpl(name, result, nameToDocItem, visited);
  }
}

function getNeighborRefNames(docItem: any): string[] {
  if (!docItem) {
    return [];
  }

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
