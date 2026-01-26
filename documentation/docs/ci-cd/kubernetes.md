---
title: Kubernetes Integration
description: Using rninja in Kubernetes build environments
tags:
  - ci-cd
  - kubernetes
---

# Kubernetes Integration

Running rninja builds in Kubernetes.

## Basic Build Job

```yaml title="build-job.yaml"
apiVersion: batch/v1
kind: Job
metadata:
  name: build-job
spec:
  template:
    spec:
      containers:
        - name: builder
          image: rust:1.75
          command: ["sh", "-c"]
          args:
            - |
              cargo install rninja
              git clone $REPO_URL /workspace
              cd /workspace
              rninja
          env:
            - name: REPO_URL
              value: "https://github.com/example/project"
      restartPolicy: Never
```

## With Persistent Cache

```yaml title="build-with-cache.yaml"
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: rninja-cache-pvc
spec:
  accessModes: [ReadWriteMany]
  resources:
    requests:
      storage: 10Gi
---
apiVersion: batch/v1
kind: Job
metadata:
  name: build-job
spec:
  template:
    spec:
      containers:
        - name: builder
          image: rust:1.75
          command: ["sh", "-c"]
          args:
            - |
              cargo install rninja
              rninja
          volumeMounts:
            - name: cache
              mountPath: /root/.cache/rninja
      volumes:
        - name: cache
          persistentVolumeClaim:
            claimName: rninja-cache-pvc
      restartPolicy: Never
```

## Remote Cache Server Deployment

```yaml title="cache-server.yaml"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rninja-cache
spec:
  replicas: 1
  selector:
    matchLabels:
      app: rninja-cache
  template:
    metadata:
      labels:
        app: rninja-cache
    spec:
      containers:
        - name: rninja-cached
          image: neullabs/rninja-cached:latest
          ports:
            - containerPort: 9999
          env:
            - name: RNINJA_SERVER_TOKENS
              valueFrom:
                secretKeyRef:
                  name: rninja-secrets
                  key: tokens
          volumeMounts:
            - name: storage
              mountPath: /data
      volumes:
        - name: storage
          persistentVolumeClaim:
            claimName: rninja-cache-storage
---
apiVersion: v1
kind: Service
metadata:
  name: rninja-cache
spec:
  selector:
    app: rninja-cache
  ports:
    - port: 9999
      targetPort: 9999
```

## Build Pod with Remote Cache

```yaml title="build-pod.yaml"
apiVersion: v1
kind: Pod
metadata:
  name: build
spec:
  containers:
    - name: builder
      image: rust:1.75
      env:
        - name: RNINJA_CACHE_REMOTE_SERVER
          value: "tcp://rninja-cache:9999"
        - name: RNINJA_CACHE_TOKEN
          valueFrom:
            secretKeyRef:
              name: rninja-secrets
              key: client-token
        - name: RNINJA_CACHE_MODE
          value: "auto"
      command: ["sh", "-c", "cargo install rninja && rninja"]
  restartPolicy: Never
```

## Secrets

```bash
kubectl create secret generic rninja-secrets \
  --from-literal=tokens="server-token" \
  --from-literal=client-token="client-token"
```
