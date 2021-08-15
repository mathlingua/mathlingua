import { DecompositionResult } from './api';

export class Search {
  //                       word  -> set of paths
  private searchIndex: Map<string, Set<string>> = new Map();

  constructor(data: DecompositionResult) {
    for (const fileResult of data.collectionResult.fileResults) {
      for (const entity of fileResult.entities) {
        for (const word of entity.words) {
          const key = word.toLowerCase();
          const pathSet = this.searchIndex.get(key) ?? new Set();
          pathSet.add(fileResult.relativePath);
          this.searchIndex.set(key, pathSet);
        }
      }
    }
  }

  search(query: string): string[] {
    const terms = query
      .split(' ')
      .map((it) => it.trim().toLowerCase())
      .filter((it) => it.length > 0);
    if (terms.length === 0) {
      return [];
    }

    let result = this.searchIndex.get(terms[0]) ?? new Set();
    if (result.size === 0) {
      return [];
    }

    for (let i = 1; i < terms.length; i++) {
      result = intersect(result, this.searchIndex.get(terms[i]) ?? new Set());
    }

    return Array.from(result);
  }
}

function intersect(set1: Set<string>, set2: Set<string>): Set<string> {
  if (set2.size > set1.size) {
    return intersect(set2, set1);
  }

  const result: Set<string> = new Set();
  for (const x of Array.from(set1.values())) {
    if (set2.has(x)) {
      result.add(x);
    }
  }
  return result;
}
