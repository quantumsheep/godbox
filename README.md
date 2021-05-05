# Godbox
Secure sandboxing system for untrusted code execution.

# Installation
### Docker Compose
```yml
version: "3"

services:
  godbox:
    image: quantumsheep/godbox
    privileged: true
    ports:
      - 8080:8080
```

### Docker
```sh
docker run -it -d --privileged -p 8080:8080 quantumsheep/godbox
```

# Usage
## POST /run
### Properties
| Name                | Description                                                                            |
|---------------------|----------------------------------------------------------------------------------------|
| compile_script      | Multi-lines bash script used in the compilation phase                                  |
| run_script*         | Multi-lines bash script used in the run phase                                          |
| files*              | Base64-encoded zip file containing the files used in the compilation and/or run phases |
| compile_environment | Environment variables used in the compilation phase (along with `compile_script`)      |
| run_environment     | Environment variables used in the run phase (along with `run_script`)                  |
| shared_environment  | Environment variables used in both the compilation and the run phases                  |

### Example with compilation
**The files should be passed as a base64 zip archive.**

The folowing demonstration uses the folowing file architecture:
```
.
├── src
    └── app.c
```

Encoded using command `zip -q -r - * | base64` (could have been a library, it doesn't matter while it keeps beeing `files -> zip -> base64`).

```json
{
	"compile_script": "/usr/local/gcc-11.1.0/bin/gcc src/main.c -o out",
	"run_script": "./out",
	"files": "UEsDBAoAAAAAAJe1pVIAAAAAAAAAAAAAAAAEABwAc3JjL1VUCQADvgOTYNQDk2B1eAsAAQT1AQAABBQAAABQSwMEFAAIAAgABbalUgAAAAAAAAAATQAAAAoAHABzcmMvbWFpbi5jVVQJAAOKBJNgjASTYHV4CwABBPUBAAAEFAAAAFPOzEvOKU1JVbApLknJzNfLsOPiyswrUchNzMzT0OSq5lIAgoLSkmINJY/UnJx8HYXw/KKcFEUlTWsusFxRaklpUZ6CgTVXLRcAUEsHCMUkHr9KAAAATQAAAFBLAQIeAwoAAAAAAJe1pVIAAAAAAAAAAAAAAAAEABgAAAAAAAAAEADtQQAAAABzcmMvVVQFAAO+A5NgdXgLAAEE9QEAAAQUAAAAUEsBAh4DFAAIAAgABbalUsUkHr9KAAAATQAAAAoAGAAAAAAAAQAAAKSBPgAAAHNyYy9tYWluLmNVVAUAA4oEk2B1eAsAAQT1AQAABBQAAABQSwUGAAAAAAIAAgCaAAAA3AAAAAAA"
}
```

#### Output
```json
{
  "compile_status": 0,
  "compile_stdout": "",
  "compile_stderr": "OK (0.044 sec real, 0.066 sec wall)\n",
  "run_status": 0,
  "run_stdout": "Hello, World!\n",
  "run_stderr": "OK (0.003 sec real, 0.014 sec wall)\n"
}
```

### Example without compilation
**The files should be passed as a base64 zip archive.**

The folowing demonstration uses the folowing file architecture:
```
.
├── src
│   └── app.js
└── package.json
```

Encoded using command `zip -q -r - * | base64` (could have been a library, it doesn't matter while it keeps beeing `files -> zip -> base64`).

```json
{
	"run_script": "/usr/local/node-14.16.1/bin/node /box",
	"files": "UEsDBBQACAAIAOq4kVIAAAAAAAAAAL4AAAAMABwAcGFja2FnZS5qc29uVVQJAAMHTntgB057YHV4CwABBPUBAAAEFAAAAEWOsQ4CIRBEe76CbK14trZW1pbGgsAmh3Jw2eU05nL/LgsmlvPmJTOr0hqSnRBOGkaMMe9T9gg74S8kDjlJdTSDGTr1yI7CXH5Nh5MNLTG5g51n8+DOu8q1WmsUUCwVMWVGG6h0a+YTP+9MXtTbvRG7lDHTfyMGh4nb08v1DGpTX1BLBwgeG7HZgQAAAL4AAABQSwMECgAAAAAA7biRUgAAAAAAAAAAAAAAAAQAHABzcmMvVVQJAAMOTntgHE57YHV4CwABBPUBAAAEFAAAAFBLAwQUAAgACACXXqJSAAAAAAAAAAAeAAAACgAcAHNyYy9hcHAuanNVVAkAA212jmBudo5gdXgLAAEE9QEAAAQUAAAAS87PK87PSdXLyU/XUPdIzcnJ11EIzy/KSVFU17TmAgBQSwcIZXvh2SAAAAAeAAAAUEsBAh4DFAAIAAgA6riRUh4bsdmBAAAAvgAAAAwAGAAAAAAAAQAAAKSBAAAAAHBhY2thZ2UuanNvblVUBQADB057YHV4CwABBPUBAAAEFAAAAFBLAQIeAwoAAAAAAO24kVIAAAAAAAAAAAAAAAAEABgAAAAAAAAAEADtQdcAAABzcmMvVVQFAAMOTntgdXgLAAEE9QEAAAQUAAAAUEsBAh4DFAAIAAgAl16iUmV74dkgAAAAHgAAAAoAGAAAAAAAAQAAAKSBFQEAAHNyYy9hcHAuanNVVAUAA212jmB1eAsAAQT1AQAABBQAAABQSwUGAAAAAAMAAwDsAAAAiQEAAAAA"
}
```

#### Output
```json
{
  "compile_status": null,
  "compile_stdout": null,
  "compile_stderr": null,
  "run_status": 0,
  "run_stdout": "Hello, World!\n",
  "run_stderr": "OK (0.003 sec real, 0.008 sec wall)\n"
}
```
