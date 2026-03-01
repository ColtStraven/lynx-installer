import type { Section, LynxProject } from '../App'
import './Sidebar.css'

interface Props {
  section: Section
  onSection: (s: Section) => void
  project: LynxProject
}

const NAV: { id: Section; label: string; icon: string; desc: string }[] = [
  { id: 'app',   label: 'App Info',  icon: '◈', desc: 'Name, version, publisher' },
  { id: 'files', label: 'Files',     icon: '⊞', desc: 'Files to bundle' },
  { id: 'steps', label: 'Steps',     icon: '◎', desc: 'Install steps' },
  { id: 'theme', label: 'Theme',     icon: '◐', desc: 'UI appearance' },
  { id: 'build', label: 'Build',     icon: '▶', desc: 'Package & export' },
]

function appInfoValid(app: LynxProject['app']): boolean {
  if (!app.name.trim() || app.name.trim().length < 2) return false
  if (!app.version.trim() || !/^\d+\.\d+(\.\d+)?$/.test(app.version.trim())) return false
  if (!app.id.trim() || !/^[a-zA-Z][a-zA-Z0-9]*(\.[a-zA-Z][a-zA-Z0-9]*){1,}$/.test(app.id.trim())) return false
  if (!app.default_install_dir.trim()) return false
  return true
}

export default function Sidebar({ section, onSection, project }: Props) {
  const fileCount  = project.files.length
  const stepCount  = project.steps.length
  const appValid   = appInfoValid(project.app)

  const sectionErrors: Partial<Record<Section, boolean>> = {
    app:   !appValid,
    files: fileCount === 0,
    steps: stepCount === 0,
  }

  return (
    <aside className="sidebar">
      <div className="sidebar-logo">
        <img src="/lynx-logo.png" alt="Lynx" className="sidebar-logo-img" />
        <div className="sidebar-logo-text">
          <span className="sidebar-logo-name">Lynx</span>
          <span className="sidebar-logo-sub">Builder</span>
        </div>
      </div>

      <div className="sidebar-project">
        <div className="sidebar-project-name">{project.app.name || 'Untitled'}</div>
        <div className="sidebar-project-version">v{project.app.version}</div>
      </div>

      <nav className="sidebar-nav">
        {NAV.map(item => (
          <button
            key={item.id}
            className={`nav-item ${section === item.id ? 'active' : ''}`}
            onClick={() => onSection(item.id)}
          >
            <span className="nav-icon">{item.icon}</span>
            <span className="nav-body">
              <span className="nav-label">{item.label}</span>
              <span className="nav-desc">{item.desc}</span>
            </span>
            {item.id === 'files' && fileCount > 0 && (
              <span className="nav-badge">{fileCount}</span>
            )}
            {item.id === 'steps' && stepCount > 0 && (
              <span className="nav-badge">{stepCount}</span>
            )}
            {sectionErrors[item.id] && (
              <span className="nav-error-dot" title="Needs attention" />
            )}
          </button>
        ))}
      </nav>

      <div className="sidebar-status">
        <div className="status-row">
          <span className={`status-dot ${appValid ? 'ok' : 'warn'}`} />
          <span>App info</span>
          <span className="status-val">{appValid ? 'ready' : 'incomplete'}</span>
        </div>
        <div className="status-row">
          <span className={`status-dot ${fileCount > 0 ? 'ok' : 'warn'}`} />
          <span>Files</span>
          <span className="status-val">{fileCount} added</span>
        </div>
        <div className="status-row">
          <span className={`status-dot ${stepCount > 0 ? 'ok' : 'err'}`} />
          <span>Steps</span>
          <span className="status-val">{stepCount} defined</span>
        </div>
      </div>

      <div className="sidebar-version">
        <span>v0.1.0</span>
      </div>
    </aside>
  )
}