//! Shared CSS — matches stormd's palette so the iframe-embedded plugin
//! looks like a first-class stormd page.

pub fn css() -> &'static str {
    r#"
* { margin: 0; padding: 0; box-sizing: border-box; }
body { background: #0f0f1a; color: #e0e0e0; font-family: -apple-system, 'Segoe UI', system-ui, sans-serif; padding: 16px 20px; }
a { color: #8be9fd; text-decoration: none; }
a:hover { text-decoration: underline; }

.subnav { display: flex; gap: 4px; margin-bottom: 16px; border-bottom: 1px solid #2a2d45; padding-bottom: 8px; }
.subnav a {
    padding: 6px 12px; border-radius: 6px; font-size: 13px; font-weight: 500;
    color: #888; transition: all 0.15s;
}
.subnav a:hover { color: #e0e0e0; background: #1e2140; text-decoration: none; }
.subnav a.active { color: #fff; background: #2a2d50; }

h1 { font-size: 20px; font-weight: 700; color: #e94560; margin-bottom: 12px; letter-spacing: -0.5px; }

.card {
    background: #16192e; border: 1px solid #2a2d45; border-radius: 8px;
    padding: 16px 20px; margin-bottom: 16px;
}
.card h2 { font-size: 14px; font-weight: 600; color: #888; margin-bottom: 12px; text-transform: uppercase; letter-spacing: 0.5px; }

table { width: 100%; border-collapse: collapse; }
th { text-align: left; padding: 10px 12px; font-size: 11px; font-weight: 600; text-transform: uppercase;
     letter-spacing: 0.5px; color: #666; border-bottom: 1px solid #2a2d45; }
td { padding: 10px 12px; font-size: 13px; border-bottom: 1px solid #1a1d32; }
tr:hover { background: #1a1d32; }

.badge {
    display: inline-block; padding: 2px 8px; border-radius: 10px; font-size: 11px;
    font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px;
}
.badge-green { background: #1a4a2a; color: #50fa7b; }
.badge-red { background: #4a1a2a; color: #e94560; }
.badge-yellow { background: #4a3a1a; color: #f1fa8c; }
.badge-cyan { background: #1a3a4a; color: #8be9fd; }
.badge-gray { background: #2a2d45; color: #888; }

.mono { font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', monospace; }

.stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 12px; margin-bottom: 16px; }
.stat-card {
    background: #16192e; border: 1px solid #2a2d45; border-radius: 8px; padding: 16px;
}
.stat-card .label { font-size: 11px; color: #666; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 4px; }
.stat-card .value { font-size: 24px; font-weight: 700; }
.stat-card .value.green { color: #50fa7b; }
.stat-card .value.cyan { color: #8be9fd; }
.stat-card .value.yellow { color: #f1fa8c; }

p.hint { color: #666; font-size: 12px; margin-top: 8px; }
code { background: #1a1d32; padding: 1px 6px; border-radius: 3px; font-size: 12px; }
"#
}

pub fn subnav(active: &str) -> String {
    let mut out = String::from("<div class=\"subnav\">");
    for (name, href) in [
        ("Overview", "/"),
        ("Registries", "/registries"),
        ("Repos", "/repos"),
        ("Users", "/users"),
        ("System", "/system"),
    ] {
        let cls = if name == active { " class=\"active\"" } else { "" };
        out.push_str(&format!("<a href=\"{href}\"{cls}>{name}</a>"));
    }
    out.push_str("</div>");
    out
}

pub fn page(title: &str, active: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<title>qregistry — {title}</title>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>{css}</style>
</head>
<body>
{subnav}
<h1>{title}</h1>
{body}
</body>
</html>"#,
        css = css(),
        subnav = subnav(active),
    )
}
