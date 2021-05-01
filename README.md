# Godbox
Secure sandboxing system for untrusted code execution.

## Installation
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
