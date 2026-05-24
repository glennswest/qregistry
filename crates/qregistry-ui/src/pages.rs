use crate::style::page;
use crate::AppState;
use axum::extract::State;
use axum::response::Html;
use qregistry_core::StorageTier;
use std::sync::Arc;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn tier_badge(t: StorageTier) -> &'static str {
    match t {
        StorageTier::Fast => r#"<span class="badge badge-cyan">fast</span>"#,
        StorageTier::Archive => r#"<span class="badge badge-yellow">archive</span>"#,
    }
}

pub async fn overview(State(state): State<Arc<AppState>>) -> Html<String> {
    let cfg = state.config.read().await;
    let body = format!(
        r#"<div class="stats-grid">
    <div class="stat-card"><div class="label">Registries</div><div class="value cyan">{regs}</div></div>
    <div class="stat-card"><div class="label">Repos</div><div class="value cyan">{repos}</div></div>
    <div class="stat-card"><div class="label">Public Repos</div><div class="value green">{public}</div></div>
    <div class="stat-card"><div class="label">Users</div><div class="value yellow">{users}</div></div>
</div>
<div class="card">
    <h2>OCI Service</h2>
    <p>Endpoint: <code class="mono">{url}</code></p>
    <p class="hint">One OCI service hosts every registry. A registry is a release/layer group (e.g. <code>4.18.4</code>) on a storage tier; within it are rspacefs repos (e.g. <code>system</code>, <code>user</code>, <code>pvc</code>). Push: <code class="mono">podman push --tls-verify=false &lt;host&gt;:5000/&lt;registry&gt;/&lt;repo&gt;:&lt;tag&gt;</code></p>
</div>
<div class="card">
    <h2>About qregistry</h2>
    <p>A Proxmox LXC appliance bundling <code>rspace_registry</code> + <code>rspacefs</code> on a Fedora base. Tiered, multi-registry OCI store for images, PVCs (host snapshot-back via rspacefs-pvc), and config artifacts — each repo its own rspacefs.</p>
</div>"#,
        regs = cfg.registries.len(),
        repos = cfg.repos.len(),
        public = cfg.repos.iter().filter(|t| t.public).count(),
        users = cfg.users.len(),
        url = esc(&cfg.oci.url),
    );
    Html(page("Overview", "Overview", &body))
}

pub async fn registries(State(state): State<Arc<AppState>>) -> Html<String> {
    let cfg = state.config.read().await;
    let rows: String = if cfg.registries.is_empty() {
        r#"<tr><td colspan="4" style="color:#666;text-align:center;padding:24px">No registries configured.</td></tr>"#.to_string()
    } else {
        cfg.registries
            .iter()
            .map(|r| {
                let repo_count = cfg.repos.iter().filter(|t| t.registry == r.name).count();
                format!(
                    r#"<tr><td class="mono">{name}</td><td>{tier}</td><td>{repos}</td><td style="color:#888">{desc}</td></tr>"#,
                    name = esc(&r.name),
                    tier = tier_badge(r.tier),
                    repos = repo_count,
                    desc = esc(&r.description),
                )
            })
            .collect()
    };
    let body = format!(
        r#"<div class="card">
    <h2>Registries</h2>
    <table>
        <thead><tr><th>Registry</th><th>Tier</th><th>Repos</th><th>Description</th></tr></thead>
        <tbody>{rows}</tbody>
    </table>
    <p class="hint">A registry pins a release/layer (e.g. <code>4.18.4</code>) to a tier: <span class="badge badge-cyan">fast</span> NVMe or <span class="badge badge-yellow">archive</span> ZFS. Migrating a registry between tiers is performed locally by rspacefs (capture/pivot); qregistry coordinates and repoints.</p>
</div>"#
    );
    Html(page("Registries", "Registries", &body))
}

pub async fn repos(State(state): State<Arc<AppState>>) -> Html<String> {
    let cfg = state.config.read().await;
    let rows: String = if cfg.repos.is_empty() {
        r#"<tr><td colspan="6" style="color:#666;text-align:center;padding:24px">No repos configured yet.</td></tr>"#.to_string()
    } else {
        cfg.repos
            .iter()
            .map(|t| {
                let public_badge = if t.public {
                    r#"<span class="badge badge-green">public</span>"#
                } else {
                    r#"<span class="badge badge-gray">private</span>"#
                };
                format!(
                    r#"<tr>
    <td class="mono">{path}</td>
    <td class="mono" style="color:#888">{registry}</td>
    <td><span class="badge badge-gray">{kind}</span></td>
    <td>{tier}</td>
    <td>{public}</td>
    <td class="mono" style="color:#888;font-size:12px">{mount}</td>
</tr>"#,
                    path = esc(&t.full_path()),
                    registry = esc(&t.registry),
                    kind = esc(t.kind.as_str()),
                    tier = tier_badge(cfg.repo_tier(t)),
                    public = public_badge,
                    mount = esc(&cfg.repo_mount(t).display().to_string()),
                )
            })
            .collect()
    };
    let body = format!(
        r#"<div class="card">
    <h2>Repos</h2>
    <table>
        <thead><tr><th>Path</th><th>Registry</th><th>Kind</th><th>Tier</th><th>Visibility</th><th>Mount (rspacefs)</th></tr></thead>
        <tbody>{rows}</tbody>
    </table>
    <p class="hint">Each repo is its own rspacefs filesystem at <code>&lt;registry&gt;/&lt;repo&gt;</code>, inheriting its registry's tier. Kinds: image, pvc (host snapshot-back), config. Per-repo placement via rspace_registry#1; dynamic CRUD in v0.2.</p>
</div>"#
    );
    Html(page("Repos", "Repos", &body))
}

pub async fn users(State(state): State<Arc<AppState>>) -> Html<String> {
    let cfg = state.config.read().await;
    let rows: String = if cfg.users.is_empty() {
        r#"<tr><td colspan="3" style="color:#666;text-align:center;padding:24px">No users configured yet.</td></tr>"#.to_string()
    } else {
        cfg.users
            .iter()
            .map(|u| {
                let admin_badge = if u.admin {
                    r#"<span class="badge badge-yellow">admin</span>"#
                } else {
                    r#"<span class="badge badge-gray">user</span>"#
                };
                let push = if u.push_tenants.is_empty() {
                    "<span style=\"color:#666\">none</span>".to_string()
                } else {
                    u.push_tenants
                        .iter()
                        .map(|t| format!(r#"<span class="badge badge-cyan">{}</span>"#, esc(t)))
                        .collect::<Vec<_>>()
                        .join(" ")
                };
                format!(
                    r#"<tr><td class="mono">{name}</td><td>{admin}</td><td>{push}</td></tr>"#,
                    name = esc(&u.username),
                    admin = admin_badge,
                    push = push,
                )
            })
            .collect()
    };
    let body = format!(
        r#"<div class="card">
    <h2>Users</h2>
    <table>
        <thead><tr><th>Username</th><th>Role</th><th>Push access</th></tr></thead>
        <tbody>{rows}</tbody>
    </table>
    <p class="hint">Passwords are bcrypt-hashed. Push access is a list of registry/repo paths. CRUD UI lands in v0.2.</p>
</div>"#
    );
    Html(page("Users", "Users", &body))
}

pub async fn system(State(state): State<Arc<AppState>>) -> Html<String> {
    let cfg = state.config.read().await;
    let body = format!(
        r#"<div class="card">
    <h2>Server</h2>
    <table>
        <tr><td>UI bind</td><td class="mono">{bind}</td></tr>
        <tr><td>Data dir</td><td class="mono">{data}</td></tr>
        <tr><td>OCI service</td><td class="mono">{url} → {listen}</td></tr>
    </table>
</div>
<div class="card">
    <h2>Version</h2>
    <p><code>qregistry v{ver}</code></p>
    <p class="hint">Built from <a href="https://github.com/glennswest/qregistry">github.com/glennswest/qregistry</a></p>
</div>"#,
        bind = esc(&cfg.server.bind),
        data = esc(&cfg.server.data_dir.display().to_string()),
        url = esc(&cfg.oci.url),
        listen = esc(&cfg.oci.listen),
        ver = env!("CARGO_PKG_VERSION"),
    );
    Html(page("System", "System", &body))
}
