# PGP-Based Authentication for 1090mb Admin Account

## Overview

The hardcoded password for the `1090mb` admin account has been removed and replaced with PGP signature-based authentication. This ensures that only the holder of the corresponding private key can access this account.

## How It Works

### Public Key
The public PGP key is stored as a constant in `src/auth/pgp.rs`:
```
-----BEGIN PGP PUBLIC KEY BLOCK-----
Version: openpgp-mobile

xsBNBGmVKw8BCAC8ooVLnnJa0CZ1QhRWK/XhycvjhNKbV9jWUMY9n6Vh06B0iWGo
wqQy7H9LjP3IMUaIcjQRJlAQJFNZxcO0d2FjTjHEeHDMGNALAhjtFwfL7r1TXTMG
BE6vSVJlrnfCnCMEyIbhT6ntCrkozKEUinz36sKl9wLoG+F7Otlo5Eg5uDttme+5
NLmX7yCFgyiv18sUCbIXVxGVOBqh4XStTtbk9bOTkd8Pe/MtqTU/pJABbihu8YxO
+LFPj7C20UcjChtOYVKan3ym1vzA/JAizh/aF/GEy295FGwdLSh4jUoHQJXKrK0+
BHLxQ1xzmCoPldpRZUuLgTV8Y3YkxjTF4XQPABEBAAHNLDEwOTBtYl9KZWVic0FJ
X2FkbWluIDwxMDkwbWJAcHJvdG9ubWFpbC5jb20+wsCKBBMBCAA+BQJplSsPCZBZ
0nyPo+Q65xYhBMiHO0V98s0hvxjeqFnSfI+j5DrnAhsDAh4BAhkBAwsJBwIVCAMW
AAICIgEAACeHCACMLzj9+XeBPvUoISnYUIIj9AxY5jJs2E86fI8mcKx3elT51qx0
UpKa8TJ9VZnFEBGSspwJ+5bFz0fwgivBZr2cKmikUjoKrJVzIuBMfYts7bU9WvXV
lMWS/jIqM4MLwqWwvYFiwSQGftMDjDUqPpg+Jakkug5mwHqbbtjtvaHA47/d3GOU
dwc9B2l1+I3cj4EkkY+SphIHCKI74jltEjvNXY8TwH+ZUssOw706i9ncCkAwdyp3
7qKxxnrFznEGpQwqXYs3bO2YhW5PlwgyNlKOX6mxQb/EocznpHJGFr0Bg2rQuRoo
LR5bpJsRQFLng3nahtZK5rCWBsNayOhtbi2OzsBNBGmVKw8BCADvcAQbOcjp9Yvr
dnJRfaTb0t4FDjPg52ueeAc/Xbqd34wYfBIqKDtkOjlGlIJaSZt8z0kCTPaHSzOZ
DorF31qPxiUlXmZUgTwb6HoTqMm9n8NobEclgpSg0BlMvvqNxYP5FyLEvyGKfW4J
jotYoecV5PsLkZThMGZunFNav6e0TiDNOWFFzwP+p8NucJqsk/yCW7MQAvacHP5A
Lhc0flZJV+La14ltgHebZ7AI2b8iOBZXtP/0mpTwdWsPOmyhUexVB+KMarvgvXBY
WN87U+62f8zurPGkQxna/Xr0118lKumj1WbClvNd5JFlSnN4SgPv2SOgms5ntNN6
J/N0umfVABEBAAHCwHYEGAEIACoFAmmVKw8JkFnSfI+j5DrnFiEEyIc7RX3yzSG/
GN6oWdJ8j6PkOucCGwwAABncB/9YzsoJ1+rmlCTh3xYkpZepbcnS2V7k1EecgNe4
7/reTWf+8XT9pkwYjbAzoZpx8uXzX6uwkykPxnhziIRu2LkbjsmnuJoIwMyXYOfm
dxLlu/2YVJDZ3yUbJzwUXDhAh1X5hQW5BfCv0AGHDWGWriWk5Fi9WYkrhomBL1tY
GxkgTnvZyTj7/QoTeqK2ko+Ww5T6wfYYKtQu4Mpm7QZCokEZR8DNYAyNJ6TIMVzL
Tadii9qkpPDIcxSITjmhbzLQbcshC2rxxo2nGD4KOuEzys7hqlU+0Tx97gonl8bx
1MB/gMBp9i1q1huZeXYczhJV/5t6KCN1WruqatUqBbcn5T2J
=xnit
-----END PGP PUBLIC KEY BLOCK-----
```

### Authentication Flow

1. **Login Endpoint**: `/api/login_pgp`
2. **Request Format**:
   ```json
   {
     "username": "1090mb",
     "signed_message": "<PGP signed message>",
     "remember_me": false
   }
   ```

3. **Message Format**: The signed message must contain:
   ```
   LOGIN:1090mb:<unix_timestamp>
   ```
   Where:
   - `LOGIN` is a constant string
   - `1090mb` is the username
   - `<unix_timestamp>` is the current Unix timestamp (must be within 5 minutes to prevent replay attacks)

### How to Login

1. Create a message with the current timestamp:
   ```bash
   echo "LOGIN:1090mb:$(date +%s)" > message.txt
   ```

2. Sign the message with your private key:
   ```bash
   gpg --clearsign --armor message.txt
   ```

3. Send the signed message to the API:
   ```bash
   curl -X POST http://localhost:8080/api/login_pgp \
     -H "Content-Type: application/json" \
     -d '{
       "username": "1090mb",
       "signed_message": "<paste the signed message here>",
       "remember_me": false
     }'
   ```

### Security Features

1. **No Password Storage**: The `1090mb` account has no password stored in the database - it's marked with `auth_type: "pgp"`
2. **Replay Attack Prevention**: Signed messages must include a timestamp within 5 minutes of the current time
3. **Rate Limiting**: Failed login attempts are rate-limited (5 attempts per IP address per 15 minutes)
4. **Signature Verification**: Only signatures from the specified public key are accepted
5. **Username Validation**: The signed message must match the username in the request

### Implementation Details

- **PGP Module**: `src/auth/pgp.rs` - Contains PGP verification logic
- **Authentication Function**: `login_pgp` in `src/auth/mod.rs` - Handles the PGP login endpoint
- **User Creation**: `ensure_pgp_user` in `src/auth/mod.rs` - Creates PGP-only users without passwords
- **Dependency**: Uses `sequoia-openpgp` library for PGP operations

### Regular Password Login Blocked

If a user tries to use the regular `/api/login` endpoint with the `1090mb` account, they will receive:
```json
{
  "error": "This account requires PGP authentication",
  "use_pgp": true
}
```
