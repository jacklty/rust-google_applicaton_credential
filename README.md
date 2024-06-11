# External Account Credential for GCP (Workload Identity Federation)
- https://google.aip.dev/auth/4117#determining-the-subject-token-in-microsoft-azure-and-url-sourced-credentials

## Service Account Keys
- URL Sourced Credential
  - Obtain the subject token for GCP STS token exchange from a URL (e.g. unauthenticated metadata endpoint on VM)
- Executable Sourced Credential
  - faciliate an executable to obtain  the subject tokens to GCP STS token 

## How to run
~~~
cargo fmt
SOURCED=ExecutableSourced cargo run

cargo build --release
SOURCED=ExecutableSourced target/release/gac
~~~