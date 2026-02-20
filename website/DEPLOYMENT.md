# GitHub Pages Deployment Guide

This guide explains how to deploy the Radium website to GitHub Pages.

## Prerequisites

1. Your repository must be public, OR
2. You must have GitHub Pages enabled for private repositories (requires GitHub Pro/Team/Enterprise)

## Setup Steps

### 1. Enable GitHub Pages

1. Go to your repository on GitHub
2. Navigate to **Settings** → **Pages**
3. Under **Source**, select:
   - **Source**: `GitHub Actions`
4. Save the settings

### 2. Verify Configuration

The website is configured in `docusaurus.config.ts`:

- **URL**: `https://radium.love` (custom domain)
- **Base URL**: `/` (root path for custom domain)
- **Organization**: `Unicorn`
- **Project**: `Radium`
- **CNAME**: `radium.love` (configured in `website/static/CNAME`)

### 3. Deployment

The website will automatically deploy when:
- You push to the `main` or `master` branch
- Changes are made to files in the `website/` directory
- You manually trigger the workflow from the Actions tab

### 4. View Your Site

After deployment, your site will be available at:
- Custom domain: `https://radium.love`

## Manual Deployment

You can also deploy manually using the Docusaurus CLI:

```bash
cd website
bun install
bun run build
bun run deploy
```

Note: The `deploy` command requires a `GITHUB_TOKEN` environment variable with appropriate permissions.

## Troubleshooting

### Build Failures

- Check the Actions tab for build errors
- Ensure all dependencies are properly installed
- Verify Node.js version (requires >= 20.0)

### 404 Errors

- Verify the `baseUrl` in `docusaurus.config.ts` matches your repository structure
- Check that GitHub Pages is enabled and using GitHub Actions as the source

### Styling Issues

- Clear browser cache
- Verify all static assets are in the `website/static/` directory

