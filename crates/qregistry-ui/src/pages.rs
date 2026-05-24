use crate::style::page;
use crate::AppState;
use axum::extract::State;
use axum::response::Html;
use std::sync::Arc;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub async fn overview(State(state): State<Arc<AppState>>) -> Html<String> {
    let cfg = state.config.read().await;
    let tenant_count = cfg.tenants.len();
    let user_count = cfg.users.len();
    let admin_count = cfg.users.iter().filter(|u| u.admin).count();
    let public_count = cfg.tenants.iter().filter(|t| t.public).count();

    let reg_rows: String = cfg
        .registries
        .iter()
        .map(|r| {
            let tier_badge = match r.tier {
                qregistry_core::StorageTier::Fast => {
                    r#"<span class="badge badge-cyan">fast</span>"#
                }
                qregistry_core::StorageTier::Archive => {
                    r#"<span class="badge badge-yellow">archive</span>"#
                }
            };
            format!(
                r#"<tr><td class="mono">{name}</td><td>{tier}</td><td class="mono">{url}</td></tr>"#,
                name = esc(&r.name),
                tier = tier_badge,
                url = esc(&r.url),
            )
        })
        .collect();

    let body = format!(
        r#"<div class="stats-grid">
    <div class="stat-card"><div class="label">Repos</div><div class="value cyan">{tenant_count}</div></div>
    <div class="stat-card"><div class="label">Public Repos</div><div class="value green">{public_count}</div></div>
    <div class="stat-card"><div class="label">Users</div><div class="value cyan">{user_count}</div></div>
    <div class="stat-card"><div class="label">Admins</div><div class="value yellow">{admin_count}</div></div>
</div>
<div class="card">
    <h2>Registry Endpoints</h2>
    <table>
        <thead><tr><th>Name</th><th>Tier</th><th>OCI endpoint</th></tr></thead>
        <tbody>{reg_rows}</tbody>
    </table>
    <p class="hint">Two registry instances on separate drives: <span class="badge badge-cyan">fast</span> on NVMe, <span class="badge badge-yellow">archive</span> on the slow ZFS drive. Push: <code class="mono">podman push --tls-verify=false &lt;host&gt;:&lt;port&gt;/&lt;repo&gt;/&lt;image&gt;:&lt;tag&gt;</code></p>
</div>
<div class="card">
    <h2>About qregistry</h2>
    <p>A Proxmox LXC appliance bundling <code>rspace_registry</code> + <code>rspacefs</code>, supervised by systemd on a Fedora base. Each repo is its own rspacefs filesystem on a tiered, separate drive.</p>
</div>"#
    );
    Html(page("Overview", "Overview", &body))
}

pub async fn tenants(State(state): State<Arc<AppState>>) -> Html<String> {
    let cfg = state.config.read().await;
    let data_dir = cfg.server.data_dir.clone();

    let rows: String = if cfg.tenants.is_empty() {
        r#"<tr><td colspan="5" style="color:#666;text-align:center;padding:24px">No repos configured yet. Add one in <code>qregistry.toml</code>.</td></tr>"#.to_string()
    } else {
        cfg.tenants
            .iter()
            .map(|t| {
                let mount = t.effective_mount_point(&data_dir);
                let public_badge = if t.public {
                    r#"<span class="badge badge-green">public</span>"#
                } else {
                    r#"<span class="badge badge-gray">private</span>"#
                };
                let tier_badge = match t.tier {
                    qregistry_core::StorageTier::Fast => {
                        r#"<span class="badge badge-cyan">fast</span>"#
                    }
                    qregistry_core::StorageTier::Archive => {
                        r#"<span class="badge badge-yellow">archive</span>"#
                    }
                };
                format!(
                    r#"<tr>
    <td class="mono">{name}</td>
    <td>{tier}</td>
    <td>{public}</td>
    <td class="mono" style="color:#888;font-size:12px">{mount}</td>
    <td style="color:#888">{desc}</td>
</tr>"#,
                    name = esc(&t.name),
                    tier = tier_badge,
                    public = public_badge,
                    mount = esc(&mount.display().to_string()),
                    desc = esc(&t.description),
                )
            })
            .collect()
    };

    let body = format!(
        r#"<div class="card">
    <h2>Repos</h2>
    <table>
        <thead><tr><th>Name</th><th>Tier</th><th>Visibility</th><th>Mount point (rspacefs)</th><th>Description</th></tr></thead>
        <tbody>{rows}</tbody>
    </table>
    <p class="hint">Each repo is a separate rspacefs filesystem. <span class="badge badge-cyan">fast</span> repos live on NVMe, <span class="badge badge-yellow">archive</span> repos on the slow ZFS drive — each tier is a separate physical drive mounted into the appliance. Mount lifecycle is managed by stormd. Dynamic add/remove lands in v0.2.</p>
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
                        .map(|t| {
                            format!(r#"<span class="badge badge-cyan">{}</span>"#, esc(t))
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                };
                format!(
                    r#"<tr>
    <td class="mono">{name}</td>
    <td>{admin}</td>
    <td>{push}</td>
</tr>"#,
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
    <p class="hint">User passwords are bcrypt-hashed. Add users via <code>qregistry.toml</code>; CRUD UI lands in v0.2.</p>
</div>"#
    );
    Html(page("Users", "Users", &body))
}

pub async fn system(State(state): State<Arc<AppState>>) -> Html<String> {
    let cfg = state.config.read().await;
    let reg_list = cfg
        .registries
        .iter()
        .map(|r| format!("{} ({}): {} → {}", r.name, r.tier, r.url, r.listen))
        .collect::<Vec<_>>()
        .join("<br>");

    let body = format!(
        r#"<div class="card">
    <h2>Server</h2>
    <table>
        <tr><td>UI bind</td><td class="mono">{bind}</td></tr>
        <tr><td>Data dir</td><td class="mono">{data}</td></tr>
        <tr><td>Registries</td><td class="mono">{reg}</td></tr>
    </table>
</div>
<div class="card">
    <h2>Version</h2>
    <p><code>qregistry v{ver}</code></p>
    <p class="hint">Built from <a href="https://github.com/glennswest/qregistry">github.com/glennswest/qregistry</a></p>
</div>"#,
        bind = esc(&cfg.server.bind),
        data = esc(&cfg.server.data_dir.display().to_string()),
        reg = reg_list,
        ver = env!("CARGO_PKG_VERSION"),
    );
    Html(page("System", "System", &body))
}
