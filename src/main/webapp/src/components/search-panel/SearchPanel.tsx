import { useEffect, useRef, useState } from 'react';
import styles from './SearchPanel.module.css';

import * as api from '../../services/api';

import { pathsUpdated, updatePathsForQuery } from '../../store/pathsSlice';
import { useDispatch } from 'react-redux';
import { queryUpdated } from '../../store/querySlice';
import { sidePanelVisibilityChanged } from '../../store/sidePanelVisibleSlice';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faSearch, faTimes } from '@fortawesome/free-solid-svg-icons';

interface Suggestion {
  complete: string;
  suffix: string;
}

export const SearchPanel = () => {
  const dispatch = useDispatch();
  const ref = useRef(null);
  const [query, setQuery] = useState('');
  const [dropdownDisplayStyle, setDropdownDisplayStyle] = useState('none');
  const [suggestions, setSuggestions] = useState([] as Suggestion[]);
  const [selectedIndex, setSelectedIndex] = useState(-1);

  function getLastQueryWord(text: string): string {
    if (!text) {
      return text;
    }
    const words = text.split(' ').filter((it: string) => it.length > 0);
    return words[words.length - 1];
  }

  function setDropDownVisible(visible: boolean) {
    setDropdownDisplayStyle(visible ? 'block' : 'none');
  }

  async function handleChange(event: any) {
    const text = event.target.value || '';
    const lastWord = getLastQueryWord(text);
    try {
      const suffixes = await api.getAutocompleteSuffixes(lastWord);
      setSuggestions(
        suffixes.map((suffix) => ({
          complete: lastWord + suffix,
          suffix,
        }))
      );
    } catch (err) {
      const message = `ERROR: ${err.message}`;
      setSuggestions([
        {
          complete: message,
          suffix: message,
        },
      ]);
    }
    setDropDownVisible(true);
    setQuery(text);
  }

  async function search(query: string) {
    dispatch(updatePathsForQuery(query));
    setDropDownVisible(false);
    dispatch(queryUpdated(query));
    dispatch(sidePanelVisibilityChanged(true));
  }

  function clearSearch() {
    setQuery('');
    setDropDownVisible(false);
    dispatch(pathsUpdated(undefined));
    dispatch(queryUpdated(''));
    dispatch(sidePanelVisibilityChanged(true));
  }

  function appendSuggestionSuffix(suffix: string) {
    const completeQuery = query.toLowerCase() + suffix.toLowerCase();
    setQuery(completeQuery);
    setDropDownVisible(false);
    return completeQuery;
  }

  useEffect(() => {
    document.addEventListener('click', (event) => {
      if (ref.current && !(ref.current as any).contains(event.target)) {
        setDropDownVisible(false);
      }
    });
  }, []);

  async function onKeyUp(e: any) {
    if (e.key === 'Escape') {
      setDropDownVisible(false);
      return;
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (dropdownDisplayStyle !== 'none') {
        const selectedSuggestion = suggestions[selectedIndex];
        let queryToUse = query;
        if (selectedSuggestion) {
          queryToUse = appendSuggestionSuffix(selectedSuggestion.suffix);
        }
        await search(queryToUse);
      } else {
        await search(query);
      }
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setDropDownVisible(true);
      setSelectedIndex(Math.min(selectedIndex + 1, suggestions.length - 1));
      return;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setDropDownVisible(true);
      setSelectedIndex(Math.max(selectedIndex - 1, 0));
      return;
    }
  }

  return (
    <span className={styles.searchArea}>
      <input
        type="text"
        id="search-input"
        className={styles.search}
        aria-label="search"
        value={query}
        onChange={handleChange}
        onKeyUp={onKeyUp}
      />
      <button
        type="button"
        className={styles.searchButton}
        onClick={() => search(query)}
      >
        <FontAwesomeIcon icon={faSearch} />
      </button>
      <button
        type="button"
        className={styles.clearButton}
        onClick={clearSearch}
      >
        <FontAwesomeIcon icon={faTimes} />
      </button>
      <div
        ref={ref}
        className={styles.searchDropdown}
        style={{ display: dropdownDisplayStyle }}
      >
        {suggestions.map((suggestion, index) => (
          <div
            key={suggestion.complete.toLowerCase()}
            onClick={() => {
              const newQuery = appendSuggestionSuffix(suggestion.suffix);
              return search(newQuery);
            }}
            className={
              index === selectedIndex
                ? styles.searchDropdownItemSelected
                : styles.searchDropdownItem
            }
          >
            <div>{suggestion.complete.toLowerCase()}</div>
          </div>
        ))}
      </div>
    </span>
  );
};
