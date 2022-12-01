## K8S PV assembler

### Installation

    helm repo add pv-assembler https://ilya-epifanov.github.io/k8s-pv-assembler/
    helm repo update

    helm install pv-assembler pv-assembler/pv-assembler --wait --create-namespace --namespace pv-assembler

### Usage

