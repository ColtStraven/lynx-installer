import { pickIconFile } from '../../dialogs'
import type { LynxProject } from '../../App'

interface Props {
  project: LynxProject
  onChange: (updater: (p: LynxProject) => LynxProject) => void
}

function fieldError(field: string, value: string): string | null {
  switch (field) {
    case 'name':
      if (!value.trim()) return 'App name is required'
      if (value.trim().length < 2) return 'Must be at least 2 characters'
      return null
    case 'version':
      if (!value.trim()) return 'Version is required'
      if (!/^\d+\.\d+(\.\d+)?$/.test(value.trim())) return 'Must be in format 1.0.0'
      return null
    case 'id':
      if (!value.trim()) return 'App ID is required'
      if (!/^[a-zA-Z][a-zA-Z0-9]*(\.[a-zA-Z][a-zA-Z0-9]*){1,}$/.test(value.trim()))
        return 'Must be reverse-domain format: com.example.myapp'
      return null
    case 'default_install_dir':
      if (!value.trim()) return 'Install directory is required'
      return null
    default:
      return null
  }
}

interface FieldProps {
  label: string
  field: string
  value: string
  onChange: (v: string) => void
  placeholder?: string
  hint?: React.ReactNode
  required?: boolean
  type?: string
}

function Field({ label, field, value, onChange, placeholder, hint, required, type = 'text' }: FieldProps) {
  const error = required ? fieldError(field, value) : null
  return (
    <div className="field">
      <label>{label}{required && ' *'}</label>
      <input
        type={type}
        value={value}
        onChange={e => onChange(e.target.value)}
        placeholder={placeholder}
        style={error ? { borderColor: 'var(--error, #ff5c5c)' } : undefined}
      />
      {error && (
        <div style={{ fontSize: 11, color: 'var(--error, #ff5c5c)', marginTop: 4 }}>
          ⚠ {error}
        </div>
      )}
      {!error && hint && <div className="token-hint">{hint}</div>}
    </div>
  )
}

export default function AppSection({ project, onChange }: Props) {
  const set = (field: string, value: string) =>
    onChange(p => ({ ...p, app: { ...p.app, [field]: value } }))

  const pickIcon = async () => {
    const path = await pickIconFile()
    if (path) set('icon', path)
  }

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
          <Field
            label="App Name" field="name" required
            value={project.app.name}
            onChange={v => set('name', v)}
            placeholder="My Application"
          />
          <Field
            label="Version" field="version" required
            value={project.app.version}
            onChange={v => set('version', v)}
            placeholder="1.0.0"
          />
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
          <Field
            label="App ID" field="id" required
            value={project.app.id}
            onChange={v => set('id', v)}
            placeholder="com.example.myapp"
            hint="Unique reverse-domain identifier"
          />
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
        <Field
          label="Default Install Directory" field="default_install_dir" required
          value={project.app.default_install_dir}
          onChange={v => set('default_install_dir', v)}
          placeholder="{program_files}/{app_name}"
          hint={
            <>
              Tokens: <span>{'{program_files}'}</span> <span>{'{local_app_data}'}</span>{' '}
              <span>{'{app_data}'}</span> <span>{'{temp}'}</span> <span>{'{app_name}'}</span>
            </>
          }
        />
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
            <label>Icon (.ico)</label>
            <div style={{ display: 'flex', gap: 8 }}>
              <input
                type="text"
                value={project.app.icon ?? ''}
                onChange={e => set('icon', e.target.value)}
                placeholder="assets/icon.ico"
                style={{ flex: 1 }}
              />
              <button className="btn btn-sm" onClick={pickIcon}>Browse</button>
            </div>
            {project.app.icon && (
              <div className="token-hint">
                Icon will be embedded in the installer .exe
              </div>
            )}
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