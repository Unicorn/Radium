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
    <div className={`${styles.filterBar} ${className || ''}`}>
      <div className={styles.filterGroup}>
        <label className={styles.filterLabel}>Category</label>
        <div className={styles.filterButtons}>
          <button
            className={`${styles.filterButton} ${!selectedCategory ? styles.filterButtonActive : ''}`}
            onClick={() => onCategoryChange(null)}>
            All
          </button>
          {categories.map((cat) => (
            <button
              key={cat.value}
              className={`${styles.filterButton} ${selectedCategory === cat.value ? styles.filterButtonActive : ''}`}
              onClick={() => onCategoryChange(cat.value)}>
              {cat.label}
              {cat.count !== undefined && <span className={styles.count}>({cat.count})</span>}
            </button>
          ))}
        </div>
      </div>

      <div className={styles.filterGroup}>
        <label className={styles.filterLabel}>Difficulty</label>
        <div className={styles.filterButtons}>
          <button
            className={`${styles.filterButton} ${!selectedDifficulty ? styles.filterButtonActive : ''}`}
            onClick={() => onDifficultyChange(null)}>
            All
          </button>
          {difficulties.map((diff) => (
            <button
              key={diff.value}
              className={`${styles.filterButton} ${selectedDifficulty === diff.value ? styles.filterButtonActive : ''}`}
              onClick={() => onDifficultyChange(diff.value)}>
              {diff.label}
              {diff.count !== undefined && <span className={styles.count}>({diff.count})</span>}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
