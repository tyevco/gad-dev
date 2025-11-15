# Godot Asset Library Submission Guide

This document outlines the process for submitting Godot Asset Browser to the official Godot Asset Library.

## Prerequisites

Before submitting to the Asset Library, ensure:

- [ ] At least one stable release (v0.1.0+) is published on GitHub
- [ ] All tests are passing (216 tests)
- [ ] Release builds are available for all platforms
- [ ] Documentation is complete and up-to-date
- [ ] CHANGELOG.md is current
- [ ] Plugin is tested in Godot 4.x

## Submission Requirements

### 1. Asset Information

**Basic Information:**
- **Asset Name**: Godot Asset Browser
- **Category**: Scripts/Plugins
- **License**: MIT (or your chosen license)
- **Repository Type**: GitHub
- **Repository URL**: https://github.com/tyevco/gad-dev
- **Issues URL**: https://github.com/tyevco/gad-dev/issues
- **Download Provider**: GitHub Release
- **Download Commit/Tag**: Latest release tag (e.g., v0.1.0)
- **Icon URL**: URL to plugin icon (256x256 PNG recommended)
- **Godot Version**: 4.0+ (specify minimum compatible version)

### 2. Description

```markdown
# Godot Asset Browser

A powerful, native Rust-based asset browser plugin for Godot 4.x that provides advanced asset management capabilities.

## Features

- **Fast Asset Search**: Search index with pre-computed data (6x faster than linear search)
- **Lazy Loading**: Pagination support for smooth browsing of large asset lists
- **Image Caching**: Automatic preview image caching with LRU eviction
- **Multiple Asset Sources**: Support for Godot Asset Library and custom repositories
- **Download Management**: Concurrent downloads with progress tracking and pause/resume
- **User Features**: Collections, history tracking, and usage statistics
- **Health Monitoring**: Automatic source health checking with circuit breaker pattern
- **Cross-Platform**: Native builds for Linux, Windows, and macOS

## Performance

- Search: 6x faster (80ms vs 500ms for 1000 assets)
- Memory: 90-100% reduction for read operations
- Page loads: 40x faster with pagination
- Image caching: Instant load for cached previews

## Installation

1. Download the appropriate build for your platform from GitHub Releases
2. Extract to your Godot project's `addons/` folder
3. Enable the plugin in Project → Project Settings → Plugins

## Requirements

- Godot 4.0 or later
- Internet connection for asset downloads

## Documentation

- User Guide: [USER_GUIDE.md](https://github.com/tyevco/gad-dev/blob/main/USER_GUIDE.md)
- API Documentation: [PERFORMANCE_OPTIMIZATION.md](https://github.com/tyevco/gad-dev/blob/main/PERFORMANCE_OPTIMIZATION.md)
- Contributing: [CONTRIBUTING.md](https://github.com/tyevco/gad-dev/blob/main/.github/CONTRIBUTING.md)

## Support

Report issues at: https://github.com/tyevco/gad-dev/issues
```

### 3. Screenshots/Preview Images

Prepare high-quality screenshots showing:

1. **Main Interface** (1920x1080 recommended)
   - Asset browser with search and filtering
   - Preview images displayed
   - Category selection

2. **Asset Details** (1920x1080)
   - Asset detail panel
   - Download progress indicator
   - Metadata display

3. **Settings/Configuration** (1920x1080)
   - Configuration options
   - Asset source management
   - User preferences

4. **Plugin Icon** (256x256 PNG)
   - Clean, recognizable icon
   - Transparent background
   - Represents asset browsing/management

### 4. Download URL

The download URL should point to the latest GitHub release:

```
https://github.com/tyevco/gad-dev/releases/latest/download/godot-asset-browser-{platform}.{ext}
```

Or use the generic release page:
```
https://github.com/tyevco/gad-dev/releases/latest
```

## Submission Process

### Step 1: Create Godot Asset Library Account

1. Visit https://godotengine.org/asset-library/asset
2. Click "Register" or log in with your existing account
3. Verify your email address

### Step 2: Submit Asset

1. Go to https://godotengine.org/asset-library/asset
2. Click "Submit Asset" button
3. Fill out the submission form with information from above
4. Upload screenshots and icon
5. Provide detailed description (use template above)
6. Submit for review

### Step 3: Wait for Review

- Asset Library moderators will review your submission
- This typically takes 1-7 days
- You'll receive an email when reviewed
- Be prepared to make changes if requested

### Step 4: Address Feedback

If moderators request changes:

1. Make necessary modifications
2. Update the submission
3. Respond to moderator comments
4. Wait for re-review

### Step 5: Publication

Once approved:

- Asset will be published to the Godot Asset Library
- Users can browse and install directly from Godot Editor
- Asset will appear in search results

## Updating the Asset

When releasing new versions:

1. Create a new GitHub release with updated version tag
2. Go to your asset page on Godot Asset Library
3. Click "Edit" button
4. Update version information
5. Update download URL if needed
6. Update screenshots if there are UI changes
7. Update description with new features
8. Submit for review (updates also require approval)

## Asset Library Best Practices

### Version Numbering

Follow Semantic Versioning (SemVer):
- **Major.Minor.Patch** (e.g., 1.0.0)
- Major: Breaking changes
- Minor: New features (backward compatible)
- Patch: Bug fixes

### Changelog

Maintain a clear CHANGELOG.md:
- List all changes for each version
- Use categories: Added, Changed, Fixed, Removed
- Help users understand what's new

### Documentation

- Keep documentation up-to-date
- Provide clear installation instructions
- Include troubleshooting section
- Add usage examples

### Support

- Respond to issues promptly
- Be helpful and patient with users
- Fix critical bugs quickly
- Consider user feedback for new features

## Automated Submission (Future)

The release.yml workflow includes a placeholder for automated Asset Library submission:

```yaml
- name: Prepare Asset Library submission
  run: |
    echo "Asset Library submission would be triggered here"
    echo "This requires manual approval and asset library credentials"
```

To enable automation:

1. Obtain Asset Library API credentials (if available)
2. Add credentials as GitHub secrets
3. Implement API submission script
4. Update workflow to call script

**Note**: As of 2024, Godot Asset Library doesn't provide a public API for automated submissions, so this remains a manual process.

## Checklist Before Submission

- [ ] All tests passing (216 tests)
- [ ] Documentation complete
  - [ ] README.md
  - [ ] USER_GUIDE.md
  - [ ] CHANGELOG.md
  - [ ] CONTRIBUTING.md
- [ ] Release builds created for all platforms
  - [ ] Linux (x86_64)
  - [ ] Windows (x86_64)
  - [ ] macOS (universal)
- [ ] GitHub release published with:
  - [ ] Version tag (e.g., v0.1.0)
  - [ ] Release notes
  - [ ] Platform-specific downloads
  - [ ] Documentation files
- [ ] Screenshots prepared
  - [ ] Main interface
  - [ ] Asset details
  - [ ] Settings
  - [ ] Plugin icon (256x256)
- [ ] Tested in Godot 4.x
  - [ ] Installation works
  - [ ] All features functional
  - [ ] No critical bugs
  - [ ] Performance acceptable
- [ ] License file included
- [ ] Asset Library account created
- [ ] Submission form ready

## Post-Submission

After your asset is published:

1. **Monitor Feedback**
   - Watch for user reports and issues
   - Respond to comments on Asset Library page
   - Engage with community

2. **Track Metrics**
   - Monitor download counts
   - Track GitHub stars and forks
   - Analyze user feedback

3. **Plan Updates**
   - Address user-requested features
   - Fix reported bugs
   - Improve based on feedback

4. **Promote**
   - Share on Godot community forums
   - Post on Reddit r/godot
   - Tweet about releases
   - Write blog posts about features

## Resources

- **Asset Library**: https://godotengine.org/asset-library/asset
- **Submission Guidelines**: https://docs.godotengine.org/en/stable/community/asset_library/submitting_to_assetlib.html
- **Best Practices**: https://docs.godotengine.org/en/stable/community/asset_library/what_is_assetlib.html
- **Community Forum**: https://forum.godotengine.org/

## Support

For questions about submission process:
- Godot Asset Library: https://godotengine.org/asset-library/asset
- Community Forum: https://forum.godotengine.org/
- Discord: https://discord.gg/godotengine

---

**Note**: This is a living document. Update it as the submission process evolves or as Godot Asset Library requirements change.
