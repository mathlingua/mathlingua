import axios from 'axios';

export interface EntityResult {
  id: string;
  type: string;
  signature: string;
  rawHtml: string;
  renderedHtml: string;
  words: string[];
}

export interface FileResult {
  relativePath: string;
  content: string;
  entities: EntityResult[];
}

export interface ErrorResult {
  relativePath: string;
  message: string;
  row: number;
  column: number;
}

export interface CollectionResult {
  fileResults: FileResult[];
  errors: ErrorResult[];
}

export interface DecompositionResult {
  homeHtml: string;
  collectionResult: CollectionResult;
}

export async function getAllPaths(): Promise<string[]> {
  const res = await axios.get('/api/allPaths');
  return res.data.paths;
}

export async function getHomeHtml(): Promise<string> {
  const res = await axios.get('/api/home');
  return res.data.homeHtml;
}

export async function getFileResult(
  path: string
): Promise<FileResult | undefined> {
  const res = await axios.get('/api/fileResult', {
    params: { path },
  });
  return res.data;
}

export async function search(query: string): Promise<string[]> {
  const res = await axios.get('/api/search', { params: { query } });
  return res.data.paths;
}

export async function getAutocompleteSuffixes(word: string): Promise<string[]> {
  const res = await axios.get('/api/completeWord', { params: { word } });
  return res.data.suffixes;
}

export async function getEntityWithSignature(
  signature: string
): Promise<EntityResult | undefined> {
  const res = await axios.get('/api/withSignature', { params: { signature } });
  return res.data;
}
