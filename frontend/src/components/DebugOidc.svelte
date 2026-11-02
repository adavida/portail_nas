<script lang="ts">
  import type { DebugInfo } from "../api/debug";

  type Props = {
    info: DebugInfo;
  };

  let { info }: Props = $props();
</script>

<dl data-testid="debug-oidc">
  <dt>issuer</dt>
  <dd>{info.issuer}</dd>
  <dt>key_bits</dt>
  <dd>{info.key_bits}</dd>
  <dt>pending_ttl_secs</dt>
  <dd>{info.pending_ttl_secs}</dd>
  <dt>code_ttl_secs</dt>
  <dd>{info.code_ttl_secs}</dd>
  <dt>session_ttl_secs</dt>
  <dd>{info.session_ttl_secs}</dd>
  <dt>access_ttl_secs</dt>
  <dd>{info.access_ttl_secs}</dd>
</dl>
