/**
 * Accessibility Testing Helpers
 *
 * Utilities for testing accessibility compliance
 */

import { Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

export interface AccessibilityOptions {
  /** WCAG tags to test against */
  tags?: string[];
  /** Rules to disable */
  disabledRules?: string[];
  /** Specific selectors to include */
  include?: string[];
  /** Specific selectors to exclude */
  exclude?: string[];
}

/**
 * Run accessibility scan on current page
 */
export async function runAccessibilityScan(
  page: Page,
  options: AccessibilityOptions = {}
) {
  const {
    tags = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'],
    disabledRules = [],
    include = [],
    exclude = [],
  } = options;

  let builder = new AxeBuilder({ page }).withTags(tags);

  if (disabledRules.length > 0) {
    builder = builder.disableRules(disabledRules);
  }

  if (include.length > 0) {
    builder = builder.include(include);
  }

  if (exclude.length > 0) {
    builder = builder.exclude(exclude);
  }

  return await builder.analyze();
}

/**
 * Check if element has visible focus indicator
 */
export async function hasVisibleFocusIndicator(page: Page): Promise<boolean> {
  return await page.evaluate(() => {
    const el = document.activeElement as HTMLElement;
    if (!el || el === document.body) return false;

    const styles = window.getComputedStyle(el);

    // Check for outline
    const hasOutline =
      styles.outline !== 'none' &&
      styles.outlineWidth !== '0px' &&
      styles.outlineColor !== 'transparent';

    // Check for box-shadow (common focus indicator)
    const hasBoxShadow = styles.boxShadow !== 'none';

    // Check for border change
    const hasBorder =
      styles.borderWidth !== '0px' && styles.borderColor !== 'transparent';

    return hasOutline || hasBoxShadow || hasBorder;
  });
}

/**
 * Get all focusable elements on the page
 */
export async function getFocusableElements(page: Page): Promise<number> {
  return await page.$$eval(
    'button:not([disabled]), a[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    elements => elements.length
  );
}

/**
 * Check heading hierarchy
 */
export async function checkHeadingHierarchy(page: Page): Promise<{
  headings: Array<{ level: number; text: string }>;
  hasH1: boolean;
  skipsLevels: boolean;
}> {
  const headings = await page.$$eval('h1, h2, h3, h4, h5, h6', elements =>
    elements.map(el => ({
      level: parseInt(el.tagName.substring(1)),
      text: el.textContent?.trim() || '',
    }))
  );

  const hasH1 = headings.some(h => h.level === 1);

  let skipsLevels = false;
  for (let i = 1; i < headings.length; i++) {
    if (headings[i].level - headings[i - 1].level > 1) {
      skipsLevels = true;
      break;
    }
  }

  return { headings, hasH1, skipsLevels };
}

/**
 * Check for landmark regions
 */
export async function checkLandmarkRegions(page: Page): Promise<{
  hasMain: boolean;
  hasNavigation: boolean;
  landmarks: string[];
}> {
  const landmarks = await page.$$eval(
    'header, nav, main, aside, footer, [role="banner"], [role="navigation"], [role="main"], [role="complementary"], [role="contentinfo"]',
    elements =>
      elements.map(el => el.tagName + (el.getAttribute('role') ? `[${el.getAttribute('role')}]` : ''))
  );

  const hasMain = landmarks.some(l => l.includes('MAIN') || l.includes('main'));
  const hasNavigation = landmarks.some(l => l.includes('NAV') || l.includes('navigation'));

  return { hasMain, hasNavigation, landmarks };
}

/**
 * Check color contrast for text elements
 */
export async function checkColorContrast(page: Page, selector: string): Promise<{
  color: string;
  backgroundColor: string;
  fontSize: string;
}> {
  return await page.$eval(selector, el => {
    const styles = window.getComputedStyle(el);
    return {
      color: styles.color,
      backgroundColor: styles.backgroundColor,
      fontSize: styles.fontSize,
    };
  });
}

/**
 * Verify all images have alt text
 */
export async function checkImagesHaveAlt(page: Page): Promise<{
  total: number;
  withAlt: number;
  decorative: number;
  missing: number;
}> {
  return await page.$$eval('img', images => {
    let withAlt = 0;
    let decorative = 0;
    let missing = 0;

    images.forEach(img => {
      const alt = img.getAttribute('alt');
      const role = img.getAttribute('role');

      if (alt === null) {
        if (role === 'presentation' || role === 'none') {
          decorative++;
        } else {
          missing++;
        }
      } else {
        withAlt++;
      }
    });

    return {
      total: images.length,
      withAlt,
      decorative,
      missing,
    };
  });
}

/**
 * Check for form label associations
 */
export async function checkFormLabels(page: Page): Promise<{
  total: number;
  labeled: number;
  unlabeled: number;
}> {
  return await page.$$eval('input, textarea, select', inputs => {
    let labeled = 0;
    let unlabeled = 0;

    inputs.forEach(input => {
      const id = input.getAttribute('id');
      const ariaLabel = input.getAttribute('aria-label');
      const ariaLabelledBy = input.getAttribute('aria-labelledby');
      const hasAssociatedLabel = id && document.querySelector(`label[for="${id}"]`);

      if (ariaLabel || ariaLabelledBy || hasAssociatedLabel) {
        labeled++;
      } else {
        unlabeled++;
      }
    });

    return {
      total: inputs.length,
      labeled,
      unlabeled,
    };
  });
}

/**
 * Navigate through focusable elements and track focus order
 */
export async function trackFocusOrder(page: Page, count: number = 5): Promise<string[]> {
  const focusOrder: string[] = [];

  for (let i = 0; i < count; i++) {
    await page.keyboard.press('Tab');

    const elementInfo = await page.evaluate(() => {
      const el = document.activeElement;
      return (
        el?.tagName +
        (el?.getAttribute('aria-label') ? `[${el.getAttribute('aria-label')}]` : '') +
        (el?.textContent?.substring(0, 20) || '')
      );
    });

    focusOrder.push(elementInfo);
  }

  return focusOrder;
}

/**
 * Check for valid ARIA roles
 */
export async function checkARIARoles(page: Page): Promise<{
  total: number;
  valid: number;
  invalid: string[];
}> {
  const validRoles = [
    'alert',
    'alertdialog',
    'application',
    'article',
    'banner',
    'button',
    'cell',
    'checkbox',
    'columnheader',
    'combobox',
    'complementary',
    'contentinfo',
    'definition',
    'dialog',
    'directory',
    'document',
    'feed',
    'figure',
    'form',
    'grid',
    'gridcell',
    'group',
    'heading',
    'img',
    'link',
    'list',
    'listbox',
    'listitem',
    'log',
    'main',
    'marquee',
    'math',
    'menu',
    'menubar',
    'menuitem',
    'menuitemcheckbox',
    'menuitemradio',
    'navigation',
    'none',
    'note',
    'option',
    'presentation',
    'progressbar',
    'radio',
    'radiogroup',
    'region',
    'row',
    'rowgroup',
    'rowheader',
    'scrollbar',
    'search',
    'searchbox',
    'separator',
    'slider',
    'spinbutton',
    'status',
    'switch',
    'tab',
    'table',
    'tablist',
    'tabpanel',
    'term',
    'textbox',
    'timer',
    'toolbar',
    'tooltip',
    'tree',
    'treegrid',
    'treeitem',
  ];

  return await page.$$eval('[role]', (elements, validRoles) => {
    const roles = elements.map(el => el.getAttribute('role')!).filter(Boolean);

    const invalid = roles.filter(role => !validRoles.includes(role));

    return {
      total: roles.length,
      valid: roles.length - invalid.length,
      invalid,
    };
  }, validRoles);
}

/**
 * Check touch target sizes (mobile accessibility)
 */
export async function checkTouchTargetSizes(
  page: Page,
  selector: string = 'button, a'
): Promise<Array<{ width: number; height: number; meetsMinimum: boolean }>> {
  const elements = await page.$$(selector);
  const sizes: Array<{ width: number; height: number; meetsMinimum: boolean }> = [];

  for (const element of elements.slice(0, 10)) {
    const box = await element.boundingBox();
    if (box) {
      sizes.push({
        width: box.width,
        height: box.height,
        meetsMinimum: box.width >= 44 && box.height >= 44,
      });
    }
  }

  return sizes;
}

/**
 * Export accessibility report as JSON
 */
export function formatAccessibilityReport(results: any): string {
  return JSON.stringify(
    {
      violations: results.violations.map((v: any) => ({
        id: v.id,
        impact: v.impact,
        description: v.description,
        nodes: v.nodes.length,
        help: v.help,
        helpUrl: v.helpUrl,
      })),
      passes: results.passes.length,
      incomplete: results.incomplete.length,
      inapplicable: results.inapplicable.length,
    },
    null,
    2
  );
}
