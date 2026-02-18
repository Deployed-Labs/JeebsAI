# GitHub Actions Setup Guide

This document explains how to set up GitHub Actions for CI/CD in the JeebsAI repository.

## CI Workflow

The CI workflow (`ci.yml`) runs automatically on every push and pull request to the `main` and `develop` branches. It performs:

- Code formatting checks (`cargo fmt`)
- Linting with Clippy (`cargo clippy`)
- Building the project
- Running tests
- Security audit with `cargo-audit`

No additional setup is required for the CI workflow.

## Deployment Workflow

The deployment workflow (`deploy.yml`) automates deployment to your VPS. It runs:

- Automatically on every push to the `main` branch
- Manually via the GitHub Actions UI (workflow_dispatch)

### Required GitHub Secrets

You need to configure the following secrets in your GitHub repository:

1. Go to your repository on GitHub
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret** and add each of the following:

#### VPS_HOST
- **Name:** `VPS_HOST`
- **Value:** Your VPS IP address or domain (e.g., `192.168.1.100` or `example.com`)

#### VPS_USER
- **Name:** `VPS_USER`
- **Value:** Your SSH username on the VPS (e.g., `ubuntu`, `deploy`, or your username)

#### VPS_SSH_KEY
- **Name:** `VPS_SSH_KEY`
- **Value:** Your private SSH key for accessing the VPS

To generate an SSH key for deployment:
```bash
# On your local machine
ssh-keygen -t ed25519 -C "github-actions-deploy" -f ~/.ssh/github_deploy_key

# Copy the public key to your VPS
ssh-copy-id -i ~/.ssh/github_deploy_key.pub your_user@your_vps_host

# Display the private key (this is what you'll paste into GitHub)
cat ~/.ssh/github_deploy_key
```

Copy the entire private key output (including `-----BEGIN OPENSSH PRIVATE KEY-----` and `-----END OPENSSH PRIVATE KEY-----`) and paste it as the `VPS_SSH_KEY` secret value.

#### VPS_DEPLOY_PATH
- **Name:** `VPS_DEPLOY_PATH`
- **Value:** The full path where JeebsAI should be deployed on your VPS (e.g., `/home/ubuntu/JeebsAI`)

### VPS Prerequisites

Before the deployment workflow can run successfully, ensure your VPS has:

1. **Rust and Cargo installed:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **System dependencies installed:**
   ```bash
   sudo apt update
   sudo apt install -y build-essential pkg-config libssl-dev sqlite3
   ```

3. **The deployment directory created:**
   ```bash
   mkdir -p /path/to/deployment/directory
   ```

4. **Systemd service configured:**
   - Run the `install.sh` script once manually on the VPS, or
   - Manually set up the systemd service following the README

5. **SSH access configured:**
   - The SSH public key from the key pair you created should be in `~/.ssh/authorized_keys`
   - The user should have sudo permissions to restart the service

6. **Sudo permissions for service management:**
   
   Add the following to your sudoers file to allow the deploy user to restart the service without a password:
   ```bash
   sudo visudo
   ```
   
   Add this line (replace `your_user` with your actual username):
   ```
   your_user ALL=(ALL) NOPASSWD: /bin/systemctl restart jeebs, /bin/systemctl status jeebs, /bin/journalctl
   ```

## Testing the Workflows

### Testing CI Locally

You can test most CI checks locally before pushing:

```bash
# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings

# Build the project
cargo build

# Run tests
cargo test

# Security audit (requires cargo-audit)
cargo install cargo-audit
cargo audit
```

### Testing Deployment

To test the deployment workflow:

1. Ensure all secrets are configured correctly
2. Push a commit to the `main` branch, or
3. Manually trigger the workflow:
   - Go to **Actions** tab in GitHub
   - Select **Deploy to VPS** workflow
   - Click **Run workflow**
   - Select the `main` branch
   - Click **Run workflow**

## Workflow Customization

### Modifying CI Checks

Edit `.github/workflows/ci.yml` to:
- Add or remove linting rules
- Change test configurations
- Add additional build targets
- Modify caching strategies

### Modifying Deployment

Edit `.github/workflows/deploy.yml` to:
- Change the deployment trigger (currently on push to `main`)
- Add pre-deployment or post-deployment steps
- Add health checks or smoke tests
- Configure backup before deployment

## Troubleshooting

### SSH Connection Issues

If deployment fails with SSH errors:

1. Verify the SSH key is correct:
   ```bash
   # Test SSH connection from your local machine
   ssh -i ~/.ssh/github_deploy_key your_user@your_vps_host
   ```

2. Check the `VPS_HOST` and `VPS_USER` secrets are correct

3. Ensure the SSH key in GitHub secrets includes the full key with headers and footers

### Build Failures on VPS

If the build fails on the VPS:

1. Check if Rust is installed: `cargo --version`
2. Ensure system dependencies are installed
3. Check available disk space: `df -h`
4. Review deployment logs in GitHub Actions

### Service Restart Failures

If the service fails to restart:

1. Check sudo permissions are configured correctly
2. Verify the service file exists: `ls -l /etc/systemd/system/jeebs.service`
3. Check service logs on VPS: `sudo journalctl -u jeebs -n 50`

## Security Best Practices

1. **Rotate SSH keys regularly** - Update the `VPS_SSH_KEY` secret periodically
2. **Use a dedicated deploy user** - Don't use root for deployments
3. **Limit sudo permissions** - Only allow specific commands needed for deployment
4. **Keep secrets secure** - Never commit secrets to the repository
5. **Review workflow runs** - Regularly check the Actions tab for any suspicious activity

## Additional Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Encrypted Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [SSH Agent for GitHub Actions](https://github.com/webfactory/ssh-agent)
