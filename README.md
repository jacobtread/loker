<h1>
    <img src="assets/loker.png" height="128px">
</h1>

![Tests Status](https://img.shields.io/github/actions/workflow/status/jacobtread/loker/tests.yml?style=for-the-badge&label=Tests)

**Loker** is a self-hosted AWS secrets manager compatible server. With the main purpose of being used for Integration and End-to-end testing use cases without requiring alternative secret backends.

Data is stored in an encrypted SQLite database using [SQLCipher](https://github.com/sqlcipher/sqlcipher). Server supports using HTTPS and enforces AWS SigV4 signing on requests.

## Quick Start (Docker)

### HTTP

The command below will start a simple docker container for **Loker** mounting the database to the
`./data` folder.

```sh
docker run -d \
  --name loker \
  -p 8080:8080 \
  -e SM_ENCRYPTION_KEY="your-encryption-key" \
  -e SM_ACCESS_KEY_ID="your-access-key-id" \
  -e SM_ACCESS_KEY_SECRET="your-access-key-secret" \
  -v ./data:/data \
  jacobtread/loker:latest
```

### HTTPS

The command below will start a simple docker container for **Loker** with HTTPS enabled. For HTTPS
you must provide your own certificate and private key.

The following command will mount the certificates from `./certs` ensure that folder contains the
certificate (`sm.cert.pem`) and private key (`sm.key.pem`) in PEM format.

```sh
docker run -d \
  --name loker \
  -p 8443:8443 \
  -e SM_ENCRYPTION_KEY="your-encryption-key" \
  -e SM_ACCESS_KEY_ID="your-access-key-id" \
  -e SM_ACCESS_KEY_SECRET="your-access-key-secret" \
  -e SM_USE_HTTPS="true" \
  -e SM_HTTPS_CERTIFICATE_PATH="/certs/sm.cert.pem" \
  -e SM_HTTPS_PRIVATE_KEY_PATH="/certs/sm.key.pem" \
  -v ./certs:/certs \
  -v ./data:/data \
  jacobtread/loker:latest
```

## Quick Start (Docker Compose)

```yaml
services:
    loker:
        image: jacobtread/loker:latest
        container_name: loker
        restart: unless-stopped
        environment:
            SM_ENCRYPTION_KEY: "your-encryption-key"
            SM_ACCESS_KEY_ID: "your-access-key-id"
            SM_ACCESS_KEY_SECRET: "your-access-key-secret"
            # Uncomment the lines below if you want to enable HTTPS
            # SM_USE_HTTPS: "true"
            # SM_HTTPS_CERTIFICATE_PATH: "/certs/sm.cert.pem"
            # SM_HTTPS_PRIVATE_KEY_PATH: "/certs/sm.key.pem"
        ports:
            # For HTTP mode
            - "8080:8080"
            # For HTTPS mode (uncomment when using HTTPS)
            # - "8443:8443"
        volumes:
            # Persistent data storage
            - ./data:/data
            # Mount certs only if using HTTPS
            # - ./certs:/certs
```

## AWS SDK

To use with the AWS Secrets Manager SDK configure your SDK config like the following

### Rust

The following snippet is an example for using **Loker** with the `aws-sdk-secretsmanager` crate:

```rust
use aws_config::{BehaviorVersion, Region, SdkConfig};
use aws_sdk_secretsmanager::config::{Credentials, SharedCredentialsProvider};

// Update credentials to match your environment variables
let credentials = Credentials::new(
    "ACCESS_KEY_ID",
    "ACCESS_KEY_SECRET",
    None,
    None,
    "sm-credentials",
);

// Adjust endpoint to match your Loker server
let endpoint_url = "http://localhost:8008";
let sdk_config = SdkConfig::builder()
    .behavior_version(BehaviorVersion::latest())
    .region(Region::from_static("us-east-1"))
    .endpoint_url(endpoint_url)
    .credentials_provider(SharedCredentialsProvider::new(credentials))
    .build()

// Construct the client
let client = aws_sdk_secretsmanager::Client::new(&sdk_config);

// ...Use the client as normal
```

### JavaScript / TypeScript

The following snippet is an example for using **Loker** with the `@aws-sdk/client-secrets-manager` npm package:

```js
import { SecretsManagerClient } from "@aws-sdk/client-secrets-manager";

// Update credentials to match your environment variables
const credentials = {
    accessKeyId: "ACCESS_KEY_ID",
    secretAccessKey: "ACCESS_KEY_SECRET",
};

// Adjust endpoint to match your Loker server
const endpoint = "http://localhost:8080";

// Construct the client
const client = new SecretsManagerClient({
    region: "us-east-1",
    endpoint,
    credentials,
});

// ...Use the client as normal
```

## Environment Variables

| Name                      | Required                                           | Description                                            |
| ------------------------- | -------------------------------------------------- | ------------------------------------------------------ |
| SM_ENCRYPTION_KEY         | Yes                                                | Encryption key to encrypt the database with            |
| SM_DATABASE_PATH          | No (Default: secrets.db)                           | Path to the file where the database should be stored   |
| SM_ACCESS_KEY_ID          | Yes                                                | Access key ID to use the server for AWS SigV4          |
| SM_ACCESS_KEY_SECRET      | Yes                                                | Access key secret to use the server for AWS SigV4      |
| SM_SERVER_ADDRESS         | No (Default: HTTP=0.0.0.0:8080 HTTPS=0.0.0.0:8443) | Socket address to bind the server to                   |
| SM_USE_HTTPS              | No (Default: false)                                | Whether to use HTTPS instead of HTTP                   |
| SM_HTTPS_CERTIFICATE_PATH | No (Default: sm.cert.pem)                          | Path to the certificate in PEM format to use for HTTPS |
| SM_HTTPS_PRIVATE_KEY_PATH | No (Default: sm.key.pem)                           | Path to the private key in PEM format to use for HTTPS |

## Implementations:

- [x] [BatchGetSecretValue](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_BatchGetSecretValue.html)
- [x] [CreateSecret](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_CreateSecret.html)
- [x] [DeleteSecret](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_DeleteSecret.html)
- [x] [DescribeSecret](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_DescribeSecret.html)
- [x] [GetRandomPassword](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_GetRandomPassword.html)
- [x] [GetSecretValue](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_GetSecretValue.html)
- [x] [ListSecrets](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_ListSecrets.html)
- [x] [ListSecretVersionIds](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_ListSecretVersionIds.html)
- [x] [PutSecretValue](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_PutSecretValue.htmls)
- [x] [RestoreSecret](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_RestoreSecret.html)
- [x] [TagResource](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_TagResource.html)
- [x] [UntagResource](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_UntagResource.html)
- [x] [UpdateSecret](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_UpdateSecret.html)
- [x] [UpdateSecretVersionStage](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_UpdateSecretVersionStage.html)

## Not Planned:

- [ ] [CancelRotateSecret](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_CancelRotateSecret.html)
- [ ] [DeleteResourcePolicy](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_DeleteResourcePolicy.html)
- [ ] [GetResourcePolicy](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_GetResourcePolicy.html)
- [ ] [PutResourcePolicy](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_PutResourcePolicy.html)
- [ ] [RemoveRegionsFromReplication](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_RemoveRegionsFromReplication.html)
- [ ] [ReplicateSecretToRegions](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_ReplicateSecretToRegions.html)
- [ ] [RotateSecret](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_RotateSecret.html)
- [ ] [StopReplicationToReplica](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_StopReplicationToReplica.html)
- [ ] [ValidateResourcePolicy](https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_ValidateResourcePolicy.html)

## Windows Build Notes

If you are building on Windows ensure you download the required prerequisites from https://wiki.openssl.org/index.php/Compilation_and_Installation#Windows
**Loker** depends on OpenSSL for the vendored SQLCipher dependency

## Disclaimer

This project is not affiliated with or endorsed by Amazon AWS.
