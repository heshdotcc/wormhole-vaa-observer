# Infrastructure

This guide assumes you will use a Kubernetes cluster to deploy the project. Still, it's useful for Docker (or podman) too.

Refer to [heshdotcc/hacklab](https://github.com/heshdotcc/hacklab) repo as a guide for working manifests that deploy MinIO, Prometheus, Grafana, Loki, Alloy, and Nvidia operator.

For the Kubernetes guide you will need to create a namespace:
```
kubectl create ns wormhole
```

## Spy

### Docker
A quick and easy way to spin-up the Spy can be just with:

```
docker run --pull=always --platform=linux/amd64 \
    -p 7073:7073 \
    --entrypoint /guardiand ghcr.io/wormhole-foundation/guardiand:latest \
    spy \
    --nodeKey /node.key \
    --spyRPC "[::]:7073" \
    --env testnet
```

As per official [Wormhole docs for setting up a local Spy](https://wormhole.com/docs/infrastructure/spy/run-spy/#__tabbed_1_2) on the testnet.

That setup will handle node key generation internally by itself.

### Kubernetes

In the case of Kubernetes, it's required to generate the key and make it accessible to the Spy pod.

Here, we will use a manifest available at [wormhole-foundation/wormhole/devnet](https://github.com/wormhole-foundation/wormhole/tree/main/devnet) dir.

To create a node key for the Spy, you can leverage the very same `guardiand` image:
```
podman run --rm \
    -v "$(pwd):/keys" \
    ghcr.io/wormhole-foundation/guardiand:latest \
    keygen /keys/node.key
```

Then, a production-grade solution would imply creating a secret with such key:

```
kubectl -n wormhole create secret generic spy-node-key --from-file=node.key=./node.key
```

And mounting it securely as per the `wormhole-spy` adjacent manifests.

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
