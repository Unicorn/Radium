import type {ReactNode} from 'react';
import type {ExampleCategory, DifficultyLevel} from '@site/src/data/examples';
import styles from './styles.module.css';

export interface FilterOption {
  value: string;
  label: string;
  count?: number;
}

export interface FilterBarProps {
  categories: FilterOption[];
  difficulties: FilterOption[];
  selectedCategory: string | null;
  selectedDifficulty: string | null;
  onCategoryChange: (category: string | null) => void;
  onDifficultyChange: (difficulty: string | null) => void;
  className?: string;
}

export default function FilterBar({
  categories,
  difficulties,
  selectedCategory,
  selectedDifficulty,
  onCategoryChange,
  onDifficultyChange,
  className,
}: FilterBarProps): ReactNode {
  return (
    <div className={`${styles.filterBar} ${className || ''}`} role="search" aria-label="Filter examples">
      <div className={styles.filterGroup} role="group" aria-labelledby="category-label">
        <span id="category-label" className={styles.filterLabel}>Category</span>
        <div className={styles.filterButtons}>
          <button
            type="button"
            className={`${styles.filterButton} ${!selectedCategory ? styles.filterButtonActive : ''}`}
            onClick={() => onCategoryChange(null)}
            aria-pressed={!selectedCategory}>
            All
          </button>
          {categories.map((cat) => (
            <button
              key={cat.value}
              type="button"
              className={`${styles.filterButton} ${selectedCategory === cat.value ? styles.filterButtonActive : ''}`}
              onClick={() => onCategoryChange(cat.value)}
              aria-pressed={selectedCategory === cat.value}>
              {cat.label}
              {cat.count !== undefined && <span className={styles.count} aria-label={`${cat.count} items`}>({cat.count})</span>}
            </button>
          ))}
        </div>
      </div>

      <div className={styles.filterGroup} role="group" aria-labelledby="difficulty-label">
        <span id="difficulty-label" className={styles.filterLabel}>Difficulty</span>
        <div className={styles.filterButtons}>
          <button
            type="button"
            className={`${styles.filterButton} ${!selectedDifficulty ? styles.filterButtonActive : ''}`}
            onClick={() => onDifficultyChange(null)}
            aria-pressed={!selectedDifficulty}>
            All
          </button>
          {difficulties.map((diff) => (
            <button
              key={diff.value}
              type="button"
              className={`${styles.filterButton} ${selectedDifficulty === diff.value ? styles.filterButtonActive : ''}`}
              onClick={() => onDifficultyChange(diff.value)}
              aria-pressed={selectedDifficulty === diff.value}>
              {diff.label}
              {diff.count !== undefined && <span className={styles.count} aria-label={`${diff.count} items`}>({diff.count})</span>}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
