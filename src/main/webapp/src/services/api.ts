import axios from 'axios';
import { AutoComplete } from './AutoComplete';
import { Search } from './Search';

export interface EntityResult {
  id: string;
  relativePath: string;
  type: string;
  signature: string;
  called: string[];
  rawHtml: string;
  renderedHtml: string;
  words: string[];
}

export interface FileResult {
  relativePath: string;
  nextRelativePath?: string;
  previousRelativePath?: string;
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

export interface SignatureIndex {
  entries: SignatureIndexEntry[];
}

export interface SignatureIndexEntry {
  id: string;
  relativePath: string;
  signature: string | undefined;
  called: string[];
}

export interface DecompositionResult {
  gitHubUrl?: string;
  collectionResult: CollectionResult;
  signatureIndex: SignatureIndex;
  configuration: Configuration;
}

export interface Configuration {
  googleAnalyticsId?: string;
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

export interface GitHubUrlResponse {
  url?: string;
}

export interface CompletionItem {
  name: string;
  parts: string[];
}

export interface Completions {
  items: CompletionItem[];
}

interface ApiClient {
  getAllPaths(): Promise<string[]>;
  getFileResult(path: string): Promise<FileResult | undefined>;
  search(query: string): Promise<string[]>;
  getAutocompleteSuffixes(word: string): Promise<string[]>;
  getEntityWithSignature(signature: string): Promise<EntityResult | undefined>;
  check(): Promise<CheckResponse>;
  getGitHubUrl(): Promise<string | undefined>;
  getFirstPath(): Promise<string>;
  getSignatureIndex(): Promise<SignatureIndex>;
  getConfiguration(): Promise<Configuration>;
}

function notifyOfError(error: string) {
  console.error(error);
}

class NetworkApiClient implements ApiClient {
  async getAllPaths(): Promise<string[]> {
    try {
      const res = await axios.get('/api/allPaths');
      return res.data.paths;
    } catch (error: any) {
      notifyOfError(error.message);
      return [];
    }
  }

  async getFileResult(path: string): Promise<FileResult | undefined> {
    try {
      const res = await axios.get('/api/fileResult', {
        params: { path },
      });
      return res.data;
    } catch (error: any) {
      notifyOfError(error.message);
      return undefined;
    }
  }

  async search(query: string): Promise<string[]> {
    try {
      const res = await axios.get('/api/search', { params: { query } });
      return res.data.paths;
    } catch (error: any) {
      notifyOfError(error.message);
      return [];
    }
  }

  async getAutocompleteSuffixes(word: string): Promise<string[]> {
    try {
      const res = await axios.get('/api/completeWord', { params: { word } });
      return res.data.suffixes;
    } catch (error: any) {
      notifyOfError(error.message);
      return [];
    }
  }

  async getEntityWithSignature(
    signature: string
  ): Promise<EntityResult | undefined> {
    try {
      const res = await axios.get('/api/withSignature', {
        params: { signature },
      });
      return res.data;
    } catch (error: any) {
      notifyOfError(error.message);
      return undefined;
    }
  }

  async check(): Promise<CheckResponse> {
    try {
      const res = await axios.get('/api/check');
      return res.data;
    } catch (error: any) {
      notifyOfError(error.message);
      return { errors: [] };
    }
  }

  async getGitHubUrl(): Promise<string | undefined> {
    try {
      const res = await axios.get('/api/gitHubUrl');
      const gitHubUrlInfo: GitHubUrlResponse = res.data;
      return gitHubUrlInfo.url;
    } catch (err: any) {
      notifyOfError(err.message);
      return undefined;
    }
  }

  async getFirstPath(): Promise<string> {
    try {
      const res = await axios.get('/api/firstPath');
      return res.data.path;
    } catch (error: any) {
      notifyOfError(error.message);
      return '';
    }
  }

  async getSignatureIndex(): Promise<SignatureIndex> {
    try {
      const res = await axios.get('/api/signatureIndex');
      return res.data;
    } catch (error: any) {
      notifyOfError(error.message);
      return { entries: [] };
    }
  }

  async getConfiguration(): Promise<Configuration> {
    try {
      const res = await axios.get('/api/configuration');
      return res.data;
    } catch (error: any) {
      notifyOfError(error.message);
      return {};
    }
  }
}

class StaticApiClient implements ApiClient {
  private searchClient: Search;
  private autoCompleteClient: AutoComplete;
  private allPaths: string[];
  private signatureToEntity: Map<string, EntityResult>;
  private pathToFileResult: Map<string, FileResult>;
  private firstFileResult: FileResult | undefined;
  private checkResponse: CheckResponse;
  private gitHubUrl: string | undefined;
  private signatureIndex: SignatureIndex;
  private configuration: Configuration;

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

    this.firstFileResult = data.collectionResult.fileResults[0];

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

    this.gitHubUrl = data.gitHubUrl;
    this.signatureIndex = data.signatureIndex;
    this.configuration = data.configuration;
  }

  async getAllPaths(): Promise<string[]> {
    return this.allPaths;
  }

  async getFileResult(path: string): Promise<FileResult | undefined> {
    if (path === '') {
      return this.firstFileResult;
    }

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

  async getGitHubUrl(): Promise<string | undefined> {
    return this.gitHubUrl;
  }

  async getFirstPath(): Promise<string> {
    return this.allPaths[0];
  }

  async getSignatureIndex(): Promise<SignatureIndex> {
    return this.signatureIndex;
  }

  async getConfiguration(): Promise<Configuration> {
    return this.configuration;
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

export async function getFirstPath(): Promise<string> {
  return getClient().getFirstPath();
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

export async function getGitHubUrl(): Promise<string | undefined> {
  return getClient().getGitHubUrl();
}

export async function getSignatureIndex(): Promise<SignatureIndex> {
  return getClient().getSignatureIndex();
}

export async function getConfiguration(): Promise<Configuration> {
  return getClient().getConfiguration();
}

export async function writeFileResult(path: string, content: string) {
  try {
    await axios.put('/api/writePage', {
      path,
      content,
    });
  } catch (error: any) {
    notifyOfError(error.message);
  }
}

export async function readPage(path: string): Promise<string> {
  try {
    const result = await axios.get('/api/readPage', {
      params: { path },
    });
    return result.data.content;
  } catch (error: any) {
    notifyOfError(error.message);
    return '';
  }
}

export async function deleteDir(path: string): Promise<void> {
  try {
    await axios.post('/api/deleteDir', {
      path,
    });
  } catch (error: any) {
    notifyOfError(error.message);
  }
}

export async function deleteFile(path: string): Promise<void> {
  try {
    await axios.post('/api/deleteFile', {
      path,
    });
  } catch (error: any) {
    notifyOfError(error.message);
  }
}

export async function renameDir(
  fromPath: string,
  toPath: string
): Promise<void> {
  try {
    await axios.post('/api/renameDir', {
      fromPath,
      toPath,
    });
  } catch (error: any) {
    notifyOfError(error.message);
    return;
  }
}

export async function renameFile(
  fromPath: string,
  toPath: string
): Promise<void> {
  try {
    await axios.post('/api/renameFile', {
      fromPath,
      toPath,
    });
  } catch (error: any) {
    notifyOfError(error.message);
  }
}

export async function newDir(path: string): Promise<void> {
  try {
    await axios.post('/api/newDir', {
      path,
    });
  } catch (error: any) {
    notifyOfError(error.message);
  }
}

export async function newFile(path: string): Promise<void> {
  try {
    await axios.post('/api/newFile', {
      path,
    });
  } catch (error: any) {
    notifyOfError(error.message);
  }
}

export async function getSignatureSuffixes(prefix: string): Promise<string[]> {
  try {
    const res = await axios.get('/api/completeSignature', {
      params: { prefix },
    });
    return res.data.suffixes;
  } catch (error: any) {
    notifyOfError(error.message);
    return [];
  }
}

export async function getCompletions(): Promise<Completions> {
  try {
    const res = await axios.get('/api/completions');
    return res.data;
  } catch (error: any) {
    notifyOfError(error.message);
    return { items: [] };
  }
}
