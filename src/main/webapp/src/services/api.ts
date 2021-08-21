import axios from 'axios';
import { AutoComplete } from './AutoComplete';
import { Search } from './Search';

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
  errors: ErrorResult[];
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

export interface CheckResponse {
  errors: CheckError[];
}

export interface CheckError {
  path: string;
  message: string;
  row: number;
  column: number;
}

interface ApiClient {
  getAllPaths(): Promise<string[]>;
  getHomeHtml(): Promise<string>;
  getFileResult(path: string): Promise<FileResult | undefined>;
  search(query: string): Promise<string[]>;
  getAutocompleteSuffixes(word: string): Promise<string[]>;
  getEntityWithSignature(signature: string): Promise<EntityResult | undefined>;
  check(): Promise<CheckResponse>;
}

class NetworkApiClient implements ApiClient {
  async getAllPaths(): Promise<string[]> {
    const res = await axios.get('/api/allPaths');
    return res.data.paths;
  }

  async getHomeHtml(): Promise<string> {
    const res = await axios.get('/api/home');
    return res.data.homeHtml;
  }

  async getFileResult(path: string): Promise<FileResult | undefined> {
    const res = await axios.get('/api/fileResult', {
      params: { path },
    });
    return res.data;
  }

  async search(query: string): Promise<string[]> {
    const res = await axios.get('/api/search', { params: { query } });
    return res.data.paths;
  }

  async getAutocompleteSuffixes(word: string): Promise<string[]> {
    const res = await axios.get('/api/completeWord', { params: { word } });
    return res.data.suffixes;
  }

  async getEntityWithSignature(
    signature: string
  ): Promise<EntityResult | undefined> {
    const res = await axios.get('/api/withSignature', {
      params: { signature },
    });
    return res.data;
  }

  async check(): Promise<CheckResponse> {
    const res = await axios.get('/api/check');
    return res.data;
  }
}

class StaticApiClient implements ApiClient {
  private searchClient: Search;
  private autoCompleteClient: AutoComplete;
  private allPaths: string[];
  private signatureToEntity: Map<string, EntityResult>;
  private homeHtml: string;
  private pathToFileResult: Map<string, FileResult>;
  private checkResponse: CheckResponse;

  constructor(data: DecompositionResult) {
    this.searchClient = new Search(data);
    this.autoCompleteClient = new AutoComplete();
    for (const fileResult of data.collectionResult.fileResults) {
      for (const entity of fileResult.entities) {
        for (const word of entity.words) {
          this.autoCompleteClient.add(word);
        }
      }
    }

    this.allPaths = data.collectionResult.fileResults.map(
      (fileResult) => fileResult.relativePath
    );

    this.signatureToEntity = new Map();
    for (const fileResult of data.collectionResult.fileResults) {
      for (const entity of fileResult.entities) {
        this.signatureToEntity.set(entity.signature, entity);
      }
    }

    this.homeHtml = data.homeHtml;

    this.pathToFileResult = new Map();
    for (const fileResult of data.collectionResult.fileResults) {
      this.pathToFileResult.set(fileResult.relativePath, fileResult);
    }

    this.checkResponse = {
      errors: data.collectionResult.errors.map((err) => ({
        column: err.column,
        row: err.row,
        path: err.relativePath,
        message: err.message,
      })),
    };
  }

  async getAllPaths(): Promise<string[]> {
    return this.allPaths;
  }

  async getHomeHtml(): Promise<string> {
    return this.homeHtml;
  }

  async getFileResult(path: string): Promise<FileResult | undefined> {
    return this.pathToFileResult.get(path);
  }

  async search(query: string): Promise<string[]> {
    return this.searchClient.search(query);
  }

  async getAutocompleteSuffixes(word: string): Promise<string[]> {
    return this.autoCompleteClient.getSuffixes(word);
  }

  async getEntityWithSignature(
    signature: string
  ): Promise<EntityResult | undefined> {
    return this.signatureToEntity.get(signature);
  }

  async check(): Promise<CheckResponse> {
    return this.checkResponse;
  }
}

let client: ApiClient | null = null;
function getClient(): ApiClient {
  if (!client) {
    const data = (window as any).MATHLINGUA_DATA;
    if (data) {
      client = new StaticApiClient(data);
    } else {
      client = new NetworkApiClient();
    }
  }
  return client;
}

export function isStatic() {
  return !!(window as any).MATHLINGUA_DATA;
}

export async function getAllPaths(): Promise<string[]> {
  return getClient().getAllPaths();
}

export async function getHomeHtml(): Promise<string> {
  return getClient().getHomeHtml();
}

export async function getFileResult(
  path: string
): Promise<FileResult | undefined> {
  return getClient().getFileResult(path);
}

export async function search(query: string): Promise<string[]> {
  return getClient().search(query);
}

export async function getAutocompleteSuffixes(word: string): Promise<string[]> {
  return getClient().getAutocompleteSuffixes(word);
}

export async function getEntityWithSignature(
  signature: string
): Promise<EntityResult | undefined> {
  return getClient().getEntityWithSignature(signature);
}

export async function check(): Promise<CheckResponse> {
  return getClient().check();
}

export async function writeFileResult(path: string, content: string) {
  await axios.put('/api/writePage', {
    path,
    content,
  });
}

export async function deleteDir(path: string): Promise<void> {
  await axios.post('/api/deleteDir', {
    path,
  });
}

export async function deleteFile(path: string): Promise<void> {
  await axios.post('/api/deleteFile', {
    path,
  });
}

export async function renameDir(
  fromPath: string,
  toPath: string
): Promise<void> {
  await axios.post('/api/renameDir', {
    fromPath,
    toPath,
  });
}

export async function renameFile(
  fromPath: string,
  toPath: string
): Promise<void> {
  await axios.post('/api/renameFile', {
    fromPath,
    toPath,
  });
}

export async function newDir(path: string): Promise<void> {
  await axios.post('/api/newDir', {
    path,
  });
}

export async function newFile(path: string): Promise<void> {
  await axios.post('/api/newFile', {
    path,
  });
}

export async function getSignatureSuffixes(prefix: string): Promise<string[]> {
  const res = await axios.get('/api/completeSignature', { params: { prefix } });
  return res.data.suffixes;
}
