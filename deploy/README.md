# Deployment Notes

## NATS Version Requirement

The repo-managed deploy artifacts pin `nats-server` to `v2.12.4-alpine`.
This satisfies the Phase 9.1 security floor of `>= v2.11.1` required to
mitigate CVE-2025-30215.

## Version Checks

Verify the pinned local-development image:

```bash
docker compose -f deploy/docker-compose.yml config | rg "nats:2.12.4-alpine"
docker run --rm nats:2.12.4-alpine --version
```

Verify the Kubernetes image after applying the manifests:

```bash
kubectl -n mister-smith get deployment nats -o jsonpath='{.spec.template.spec.containers[0].image}'
kubectl -n mister-smith exec deploy/nats -- nats-server --version
```

## Permission Audit

Before shipping NATS auth changes, scan repo-managed config files for forbidden
wildcard permissions:

```bash
python3 scripts/audit_nats_permissions.py deploy
```

The audit fails on wildcard `>` and `$JS.>` permissions. Documentation and spec
Markdown are excluded on purpose so example snippets do not produce false
positives.
