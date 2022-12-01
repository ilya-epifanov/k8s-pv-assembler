cluster_name := "pv-assembler"
docker_user := "smartislav"
docker_image := "k8s-pv-assembler"

cluster-up:
    kind create cluster --name {{cluster_name}} --image kindest/node:v1.23.5 --config kind-config.yaml
    sleep "1"
    kubectl wait --namespace kube-system --for=condition=ready pod --selector="tier=control-plane" --timeout=180s

build:
    docker build --network=host -t {{docker_user}}/{{docker_image}} .

load:
    kind --name {{cluster_name}} load docker-image {{docker_user}}/{{docker_image}}:latest

deploy: load
    kubectl apply -f deploy/pv.yaml
    helm install pv-assembler charts/pv-assembler --values deploy/debug-values.yaml --wait

debug:
    kubectl apply -f deploy/debug.yaml

cluster-down:
    kind delete cluster --name {{cluster_name}}

all: cluster-up build load deploy

delete:
    kubectl delete -f deploy/debug.yaml || /bin/true
    helm delete pv-assembler --wait || /bin/true
    kubectl delete -f deploy/pv.yaml || /bin/true
