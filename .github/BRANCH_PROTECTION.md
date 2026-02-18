# Branch Protection Configuration

This document explains how to configure GitHub branch protection rules to ensure that the CI/CD pipeline blocks merging when tests fail.

## Required Branch Protection Settings

To enable the CI/CD pipeline to block merging on failed tests, configure the following branch protection rules for your `main` and `develop` branches:

### 1. Navigate to Repository Settings

1. Go to your repository on GitHub
2. Click on **Settings** tab
3. Select **Branches** from the left sidebar
4. Click **Add rule** (or edit existing rule)

### 2. Configure Branch Protection Rule

**Branch name pattern**: `main` (create separate rules for `develop` if needed)

#### Required Settings:

- ✅ **Require a pull request before merging**
  - ✅ Require approvals: 1 (recommended)
  - ✅ Dismiss stale PR approvals when new commits are pushed
  - ✅ Require review from code owners (if CODEOWNERS file exists)

- ✅ **Require status checks to pass before merging**
  - ✅ Require branches to be up to date before merging
  - **Required status checks** (select these from the list):
    - `Test Suite (stable)`
    - `Test Suite (beta)` 
    - `Security Audit`
    - `Code Coverage`

- ✅ **Require conversation resolution before merging**

- ✅ **Require signed commits** (optional but recommended)

- ✅ **Include administrators** (recommended for consistency)

- ✅ **Restrict pushes that create files** (optional)

### 3. Additional Recommendations

#### Auto-merge Protection
- Consider enabling **"Require linear history"** to keep the git history clean
- Enable **"Delete head branches automatically"** to clean up PR branches

#### Ruleset Alternative (Beta)
GitHub now offers Rulesets as a more flexible alternative to branch protection rules. Consider using rulesets for more granular control.

## Status Check Details

The CI pipeline creates the following status checks that will block merging if they fail:

| Check Name | Purpose | Blocking |
|------------|---------|----------|
| `Test Suite (stable)` | Run all tests with stable Rust | ✅ Yes |
| `Test Suite (beta)` | Run all tests with beta Rust | ✅ Yes |
| `Security Audit` | Check for known vulnerabilities | ✅ Yes |
| `Code Coverage` | Generate coverage report | ❌ No (informational) |
| `Check Dependencies` | Check for outdated deps | ❌ No (informational) |
| `Build Docker Image` | Build Docker image | ❌ No (only on main) |

## Testing the Configuration

To verify that branch protection is working correctly:

1. Create a test branch with intentionally failing tests
2. Open a pull request to `main`
3. Verify that the "Merge pull request" button is disabled
4. Check that status checks appear and show failures
5. Fix the tests and verify that merging becomes available

## Environment Variables for CI

The CI pipeline expects certain environment variables. For private repositories, you may need to add these as repository secrets:

- `DATABASE_URL` - Set automatically by CI for testing
- `STRAVA_CLIENT_ID` - Should be added as repository secret if needed
- `STRAVA_CLIENT_SECRET` - Should be added as repository secret if needed  
- `JWT_SECRET` - Set automatically by CI for testing

## Troubleshooting

### Status Checks Not Appearing
1. Ensure the workflow files are in the `main` branch
2. Check that workflows have run at least once
3. Verify workflow permissions in repository settings

### Tests Failing in CI but Passing Locally
1. Check environment variable differences
2. Verify PostgreSQL version compatibility  
3. Review caching issues - try clearing cache
4. Check for race conditions in tests

### Merge Button Still Enabled Despite Failures
1. Verify branch protection rules are configured correctly
2. Check that the exact status check names match the workflow job names
3. Ensure "Require status checks to pass" is enabled
4. Verify the rule applies to the target branch

## Additional Security Measures

Consider implementing these additional security measures:

1. **CODEOWNERS file** - Require specific people to review certain files
2. **Required reviewers** - Ensure senior developers review critical changes
3. **Signed commits** - Verify commit authenticity
4. **Dependabot** - Automated security updates for dependencies

## Monitoring and Alerts

Set up notifications for:
- Failed CI runs
- Security vulnerability discoveries
- Dependency update availability
- Unusual merge patterns

This ensures the team stays informed about the repository's health and security status.