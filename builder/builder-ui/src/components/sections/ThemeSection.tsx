import type { LynxProject } from '../../App'

interface Props {
  project: LynxProject
  onChange: (updater: (p: LynxProject) => LynxProject) => void
}

export default function ThemeSection({ project, onChange }: Props) {
  const set = (field: string, value: string) =>
    onChange(p => ({ ...p, theme: { ...p.theme, [field]: value } }))

  const accent = project.theme.accent_color || '#FF6B35'
  const bg     = project.theme.background_color || '#1A1A2E'

  return (
    <div>
      <div className="section-header">
        <div className="section-tag">04 — Theme</div>
        <h1 className="section-title">Installer appearance</h1>
        <p className="section-desc">Customize colors and branding for the installer window.</p>
      </div>

      <div className="card">
        <div className="card-title">Colors</div>
        <div className="field-row">
          <div className="field">
            <label>Accent Color</label>
            <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
              <input
                type="color"
                value={accent}
                onChange={e => set('accent_color', e.target.value)}
                style={{ width: 48, flexShrink: 0 }}
              />
              <input
                type="text"
                value={accent}
                onChange={e => set('accent_color', e.target.value)}
                placeholder="#FF6B35"
              />
            </div>
          </div>
          <div className="field">
            <label>Background Color</label>
            <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
              <input
                type="color"
                value={bg}
                onChange={e => set('background_color', e.target.value)}
                style={{ width: 48, flexShrink: 0 }}
              />
              <input
                type="text"
                value={bg}
                onChange={e => set('background_color', e.target.value)}
                placeholder="#1A1A2E"
              />
            </div>
          </div>
        </div>
      </div>

      <div className="card">
        <div className="card-title">Preview</div>
        <div style={{
          background: bg,
          borderRadius: 8,
          padding: 24,
          display: 'flex',
          flexDirection: 'column',
          gap: 12,
          border: '1px solid rgba(255,255,255,0.06)',
        }}>
          <div style={{
            fontFamily: 'Syne, sans-serif',
            fontSize: 20,
            fontWeight: 800,
            background: `linear-gradient(135deg, ${accent}, #C850C0)`,
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent',
          }}>
            {project.app.name || 'My App'}
          </div>
          <div style={{ height: 6, borderRadius: 99, background: `linear-gradient(90deg, ${accent}, #C850C0)`, width: '60%' }} />
          <div style={{
            padding: '10px 20px',
            background: `linear-gradient(135deg, ${accent}, #C850C0)`,
            borderRadius: 6,
            color: '#fff',
            fontWeight: 700,
            fontSize: 13,
            display: 'inline-flex',
            alignItems: 'center',
            gap: 8,
            alignSelf: 'flex-start',
          }}>
            Install Now →
          </div>
        </div>
      </div>

      <div className="card">
        <div className="card-title">Theme File</div>
        <div className="field-row">
          <div className="field">
            <label>Theme Name</label>
            <input
              type="text"
              value={project.theme.name}
              onChange={e => set('name', e.target.value)}
              placeholder="lynx-default"
            />
          </div>
          <div className="field">
            <label>Theme Path</label>
            <input
              type="text"
              value={project.theme.path}
              onChange={e => set('path', e.target.value)}
              placeholder="themes/lynx-default"
            />
          </div>
        </div>
      </div>
    </div>
  )
}
