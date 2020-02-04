You'll need to have an existing Kubernetes cluster and [storage class](https://kubernetes.io/docs/concepts/storage/storage-classes/).
Setting this up will vary depending on your provider.
To try it locally, you can use [MicroK8s](https://microk8s.io/) or [Minikube](https://kubernetes.io/docs/tasks/tools/install-minikube/).

Once you have a working cluster, edit the environment variables and volume sizes in `docker/k8s/*.yml`.
You may also want to change the service types to use `LoadBalancer`s depending on where you're running your cluster (add `type: LoadBalancer` to `ports)`, or `NodePort`s.
By default they will use `ClusterIP`s, which will allow access only within the cluster. See the [docs](https://kubernetes.io/docs/concepts/services-networking/service/) for more on networking in Kubernetes.

**Important** Running a database in Kubernetes will work, but is generally not recommended.
If you're deploying on any of the common cloud providers, you should consider using their managed database service instead (RDS, Cloud SQL, Azure Databse, etc.).

Now you can deploy:

```bash
# Add `-n foo` if you want to deploy into a specific namespace `foo`;
# otherwise your resources will be created in the `default` namespace.
kubectl apply -f docker/k8s/db.yml
kubectl apply -f docker/k8s/pictshare.yml
kubectl apply -f docker/k8s/lemmy.yml
```

If you used a `LoadBalancer`, you should see it in your cloud provider's console.
