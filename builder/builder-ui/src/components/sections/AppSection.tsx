import type { LynxProject } from '../../App'

interface Props {
  project: LynxProject
  onChange: (updater: (p: LynxProject) => LynxProject) => void
}

export default function AppSection({ project, onChange }: Props) {
  const set = (field: string, value: string) =>
    onChange(p => ({ ...p, app: { ...p.app, [field]: value } }))

  return (
    <div>
      <div className="section-header">
        <div className="section-tag">01 — App Info</div>
        <h1 className="section-title">Application metadata</h1>
        <p className="section-desc">Basic information about the app being installed.</p>
      </div>

      <div className="card">
        <div className="card-title">Identity</div>
        <div className="field-row">
          <div className="field">
            <label>App Name *</label>
            <input
              type="text"
              value={project.app.name}
              onChange={e => set('name', e.target.value)}
              placeholder="My Application"
            />
          </div>
          <div className="field">
            <label>Version *</label>
            <input
              type="text"
              value={project.app.version}
              onChange={e => set('version', e.target.value)}
              placeholder="1.0.0"
            />
          </div>
        </div>
        <div className="field-row">
          <div className="field">
            <label>Publisher</label>
            <input
              type="text"
              value={project.app.publisher}
              onChange={e => set('publisher', e.target.value)}
              placeholder="Acme Corp"
            />
          </div>
          <div className="field">
            <label>App ID *</label>
            <input
              type="text"
              value={project.app.id}
              onChange={e => set('id', e.target.value)}
              placeholder="com.example.myapp"
            />
            <div className="token-hint">Unique reverse-domain identifier</div>
          </div>
        </div>
        <div className="field">
          <label>Description</label>
          <textarea
            value={project.app.description}
            onChange={e => set('description', e.target.value)}
            placeholder="A short description of what this app does..."
          />
        </div>
      </div>

      <div className="card">
        <div className="card-title">Install Location</div>
        <div className="field">
          <label>Default Install Directory</label>
          <input
            type="text"
            value={project.app.default_install_dir}
            onChange={e => set('default_install_dir', e.target.value)}
            placeholder="{program_files}/{app_name}"
          />
          <div className="token-hint">
            Tokens: <span>{'{program_files}'}</span> <span>{'{local_app_data}'}</span> <span>{'{app_data}'}</span> <span>{'{temp}'}</span> <span>{'{app_name}'}</span>
          </div>
        </div>
      </div>

      <div className="card">
        <div className="card-title">Optional</div>
        <div className="field-row">
          <div className="field">
            <label>Website URL</label>
            <input
              type="url"
              value={project.app.url ?? ''}
              onChange={e => set('url', e.target.value)}
              placeholder="https://example.com"
            />
          </div>
          <div className="field">
            <label>Icon Path</label>
            <input
              type="text"
              value={project.app.icon ?? ''}
              onChange={e => set('icon', e.target.value)}
              placeholder="assets/icon.ico"
            />
          </div>
        </div>
      </div>

      <div className="card">
        <div className="card-title">Uninstaller</div>
        <div className="field-row">
          <div className="field">
            <label>
              <input
                type="checkbox"
                checked={project.uninstall.enabled}
                onChange={e => onChange(p => ({ ...p, uninstall: { ...p.uninstall, enabled: e.target.checked } }))}
                style={{ marginRight: 6 }}
              />
              Register uninstaller in Windows
            </label>
          </div>
          <div className="field">
            <label>
              <input
                type="checkbox"
                checked={project.uninstall.remove_install_dir}
                onChange={e => onChange(p => ({ ...p, uninstall: { ...p.uninstall, remove_install_dir: e.target.checked } }))}
                style={{ marginRight: 6 }}
              />
              Remove install directory on uninstall
            </label>
          </div>
        </div>
      </div>
    </div>
  )
}
