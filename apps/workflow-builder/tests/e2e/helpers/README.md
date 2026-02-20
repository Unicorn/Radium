# Test Helpers

## Console Error Capture

The `console-errors.ts` helper provides utilities for capturing console errors and page errors during Playwright tests.

### Usage

```typescript
import { setupConsoleErrorCapture } from './helpers/console-errors';

test('my test', async ({ page }) => {
  // Capture errors (won't fail test)
  const errorCollector = setupConsoleErrorCapture(page, false);
  
  await page.goto('/some-page');
  
  // Check for errors
  if (errorCollector.hasErrors()) {
    console.warn('Errors detected:', errorCollector.getSummary());
  }
  
  // Or fail test on errors
  const strictCollector = setupConsoleErrorCapture(page, true);
  // Test will fail if any console errors occur
});
```

### Options

- `failOnError: false` - Capture errors but don't fail tests (default)
- `failOnError: true` - Fail tests immediately when errors are detected

### What Gets Captured

- Console errors (`console.error()`)
- Unhandled page errors (`window.onerror`)
- Failed network requests
- Unhandled promise rejections

### Error Information

Each captured error includes:
- Type (console or pageerror)
- Message
- Timestamp
- Stack trace (for page errors)
- Console arguments (for console errors)

