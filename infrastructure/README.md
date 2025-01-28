# Infrastructure

This guide assumes you will use a Kubernetes cluster to deploy the project. Still, it's useful for Docker (or podman) too.

For the Kubernetes guide you will need to create a namespace:
```
kubectl create ns wormhole
```

## Spy

A quick and easy way to spin-up the Spy can be just with:
```
podman run --rm \
    -v "$(pwd):/keys" \
    ghcr.io/wormhole-foundation/guardiand:latest \
    keygen /keys/node.key
```
It will handle the Key generation on itself.

A more production-grade solution would imply creating a Key with `openssl rand -hex 16` and sourcing it within a secure volume:

```
kubectl -n wormhole create secret generic spy-node-key --from-file=node.key=./node.key
```

Then you can adjust the `wormhole-spy` manifest and apply if:
```
kubectl -n wormhole apply -f wormhole-spy.yaml
```

Note that, interestingly, after successfully deploying the Spy there will be a metrics endopoint available at:
```
http://$YOUR_CLUSTER_HOST:$NODEPORT/metrics
```

## Backend

The Rust-based VAAs backend that ingest data from the Spy could be deployed by applying the folder:
```
kubectl apply -R -f backend
```

The following command can help you validate that you can reach the Spy through gRPC:
```
docker run -v $(pwd)/proto:/protos \
  fullstorydev/grpcurl \
  -import-path /protos \
  -proto /protos/spy/v1/spy.proto \
  -plaintext \
  $YOUR_CLUSTER_HOST:$NODEPORT \
  spy.v1.SpyRPCService/SubscribeSignedVAA
```
Note that using payload to query its data may not work, since it doesn't support Reflection API.
