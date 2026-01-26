---
title: Jenkins Integration
description: Using rninja with Jenkins
tags:
  - ci-cd
  - jenkins
---

# Jenkins Integration

Complete guide to using rninja with Jenkins pipelines.

## Basic Pipeline

```groovy title="Jenkinsfile"
pipeline {
    agent any

    stages {
        stage('Install rninja') {
            steps {
                sh 'cargo install rninja'
            }
        }

        stage('Build') {
            steps {
                sh 'rninja'
            }
        }
    }
}
```

## With Remote Cache

```groovy title="Jenkinsfile"
pipeline {
    agent any

    environment {
        RNINJA_CACHE_REMOTE_SERVER = credentials('rninja-cache-server')
        RNINJA_CACHE_TOKEN = credentials('rninja-cache-token')
        RNINJA_CACHE_MODE = 'auto'
    }

    stages {
        stage('Build') {
            steps {
                sh '''
                    cargo install rninja
                    rninja -j0
                    rninja -t cache-stats
                '''
            }
        }
    }
}
```

## CMake Project

```groovy title="Jenkinsfile"
pipeline {
    agent {
        docker { image 'ubuntu:22.04' }
    }

    stages {
        stage('Setup') {
            steps {
                sh '''
                    apt-get update
                    apt-get install -y cmake ninja-build build-essential curl
                    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                    . $HOME/.cargo/env
                    cargo install rninja
                '''
            }
        }

        stage('Configure') {
            steps {
                sh 'cmake -G Ninja -B build -DCMAKE_BUILD_TYPE=Release'
            }
        }

        stage('Build') {
            steps {
                sh '. $HOME/.cargo/env && rninja -C build'
            }
        }

        stage('Test') {
            steps {
                sh 'cd build && ctest --output-on-failure'
            }
        }
    }
}
```

## Credentials Setup

In Jenkins > Manage Jenkins > Manage Credentials:

1. Add `rninja-cache-server` (Secret text): `tcp://cache.internal:9999`
2. Add `rninja-cache-token` (Secret text): `your-token`
