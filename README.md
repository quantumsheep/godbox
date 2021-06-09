# Godbox
Secure sandboxing system for untrusted code execution.

It uses [isolate](https://github.com/ioi/isolate) which uses specific functionnalities of the Linux kernel, thus godbox not able to run properly outside of Linux.

# Installation
## Docker Compose
```yml
version: "3"

services:
  godbox:
    image: quantumsheep/godbox:2
    privileged: true
    ports:
      - 8080:8080
```

## Docker
```sh
docker run -it -d --privileged -p 8080:8080 quantumsheep/godbox:2
```

# Environment variables
| Name                    | Type      | Default | Description                 |
|-------------------------|-----------|---------|-----------------------------|
| ALLOW_PROFILING         | `boolean` | true    | Enable or disable profiling |
| MAX_RUN_TIME_LIMIT      | `number`  | -1      | Maximum run time limit      |
| MAX_EXTRA_TIME_LIMIT    | `number`  | -1      | Maximum extra time limit    |
| MAX_WALL_TIME_LIMIT     | `number`  | -1      | Maximum wall time limit     |
| MAX_STACK_SIZE_LIMIT    | `number`  | -1      | Maximum stack size limit    |
| MAX_PROCESS_COUNT_LIMIT | `number`  | -1      | Maximum process count limit |
| MAX_MEMORY_LIMIT        | `number`  | -1      | Maximum memory limit        |
| MAX_STORAGE_LIMIT       | `number`  | -1      | Maximum storage limit       |

# Run commands
Send a `POST` HTTP request to `http://localhost:8080/run` containing the wanted configuration in JSON. See below for properties.

## Properties
| Name             | Type                     | Description                                                     |
|------------------|--------------------------|-----------------------------------------------------------------|
| phases*          | `Phase[]`                | Execution phases (check examples bellow)                        |
| files*           | `string`                 | Base64-encoded zip file containing the files used in the phases |
| environment      | `Record<string, string>` | Environment variables used in all phases                        |
| sandbox_settings | `SandboxSettings`        | Override default sandbox limitation settings                    |

### Phase
| Name             | Type                     | Default       | Description                                                                                                                         |
|------------------|--------------------------|---------------|-------------------------------------------------------------------------------------------------------------------------------------|
| name             | `string`                 | Phase's index | Name that will be used in result output                                                                                             |
| script*          | `string`                 |               | Multi-line bash script that will be executed inside the isolated environment                                                        |
| environment      | `Record<string, string>` |               | Environment variables available inside `script` execution. This will override global environment variables with the same given keys |
| sandbox_settings | `SandboxSettings`        |               | Overrides default sandbox limitation settings. This will override global sandbox settings with the same given keys                  |
| profiling        | `boolean`                | false         | Run a profiler on `script`. This functionnality is WIP                                                                              |

### SandboxSettings
| Name                | Type     | Default | Description                                                                                                                                                                                                                                                                                                                                                                                    |
|---------------------|----------|---------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| run_time_limit      | `number` | 5       | Limit run time of the whole control group in seconds. Fractional numbers are allowed                                                                                                                                                                                                                                                                                                           |
| extra_time_limit    | `number` | 0       | When a time limit is exceeded, wait for extra time seconds before killing the program. This has the advantage that the real execution time is reported, even though it slightly exceeds the limit. Fractional numbers are again allowed                                                                                                                                                        |
| wall_time_limit     | `number` | 10      | Limit wall-clock time to time seconds. Fractional values are allowed. This clock measures the time from the start of the program to its exit, so it does not stop when the program has lost the CPU or when it is for an external event. It is recommend to use `run_time_limit` as the main limit, but set `wall_time_limit` to a much higher value as a precaution against sleeping programs |
| stack_size_limit    | `number` | 128000  | Limit process stack to size kilobytes. It is subject to `memory_limit`                                                                                                                                                                                                                                                                                                                         |
| process_count_limit | `number` | 120     | Permit the program to create up to max processes and/or threads                                                                                                                                                                                                                                                                                                                                |
| memory_limit        | `number` | 512000  | Limit total memory usage by the whole control group in kilobytes                                                                                                                                                                                                                                                                                                                               |
| storage_limit       | `number` | 10240   | Limit size of files created (or modified) by the program in kilobytes                                                                                                                                                                                                                                                                                                                          |

## Example
**The files should be passed as a base64 zip archive.**

The folowing demonstration uses the folowing file architecture:
```
.
└── src
    └── main.c
```

Encoded using command `zip -q -r - * | base64` (could have been a library, it doesn't matter while it keeps beeing `files -> zip -> base64`).

```json
{
  "phases": [
    {
      "name": "Compilation",
      "script": "/usr/local/gcc-11.1.0/bin/gcc src/main.c -o out",
      "sandbox_settings": {
        "run_time_limit": 20,
        "wall_time_limit": 40
      }
    },
    {
      "name": "Execution",
      "script": "./out"
    }
  ],
  "environment": {
    "ENABLE_AWESOME_SHEEP": "true"
  },
  "files": "UEsDBAoAAAAAAJe1pVIAAAAAAAAAAAAAAAAEABwAc3JjL1VUCQADvgOTYNQDk2B1eAsAAQT1AQAABBQAAABQSwMEFAAIAAgABbalUgAAAAAAAAAATQAAAAoAHABzcmMvbWFpbi5jVVQJAAOKBJNgjASTYHV4CwABBPUBAAAEFAAAAFPOzEvOKU1JVbApLknJzNfLsOPiyswrUchNzMzT0OSq5lIAgoLSkmINJY/UnJx8HYXw/KKcFEUlTWsusFxRaklpUZ6CgTVXLRcAUEsHCMUkHr9KAAAATQAAAFBLAQIeAwoAAAAAAJe1pVIAAAAAAAAAAAAAAAAEABgAAAAAAAAAEADtQQAAAABzcmMvVVQFAAO+A5NgdXgLAAEE9QEAAAQUAAAAUEsBAh4DFAAIAAgABbalUsUkHr9KAAAATQAAAAoAGAAAAAAAAQAAAKSBPgAAAHNyYy9tYWluLmNVVAUAA4oEk2B1eAsAAQT1AQAABBQAAABQSwUGAAAAAAIAAgCaAAAA3AAAAAAA"
}
```

#### Output
```json
{
  "phases": [
    {
      "name": "Compilation",
      "status": 0,
      "stdout": "",
      "stderr": "OK (0.041 sec real, 0.048 sec wall)\n"
    },
    {
      "name": "Execution",
      "status": 0,
      "stdout": "Hello, World!\n",
      "stderr": "OK (0.001 sec real, 0.005 sec wall)\n"
    }
  ]
}
```
