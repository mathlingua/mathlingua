export type Amount = 'One' | 'ZeroOrOne' | 'ZeroOrMore' | 'OneOrMore';

export interface DocItem {
  name: string;
  oneLineDesc?: string;
  fullDesc?: string;
  example?: string;
}

export interface TerminalDocItem extends DocItem {
  form: string;
}

export interface UnionDocItem extends DocItem {
  allowed: string[]; // names of the allowed items
}

export function ref(name: string) {
  return `<${name}>`;
}

export function unref(ref: string) {
  return ref.substring(1, ref.length - 1);
}

export function literal(text: string) {
  return ` \`${text}\` `;
}

export function zeroOrOneTime() {
  return '? ';
}

export function zeroOrMoreTimes() {
  return '* ';
}

export function oneOrMoreTimes() {
  return '+ ';
}

export function group(form: string) {
  return ` (${form} )`;
}
