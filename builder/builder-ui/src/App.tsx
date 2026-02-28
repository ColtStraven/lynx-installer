import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { saveLynxFile, pickLynxFile } from './dialogs'
import Sidebar from './components/Sidebar'
import AppSection from './components/sections/AppSection'
import FilesSection from './components/sections/FilesSection'
import StepsSection from './components/sections/StepsSection'
import ThemeSection from './components/sections/ThemeSection'
import BuildSection from './components/sections/BuildSection'
import Titlebar from './components/Titlebar'
import './App.css'

export type Section = 'app' | 'files' | 'steps' | 'theme' | 'build'

export interface LynxProject {
  app: {
    name: string
    version: string
    publisher: string
    id: string
    description: string
    icon: string | null
    default_install_dir: string
    url: string | null
  }
  theme: {
    name: string
    path: string
    accent_color: string
    background_color: string
    custom_vars: Record<string, string>
  }
  files: FileEntry[]
  steps: InstallStep[]
  prerequisites: unknown[]
  uninstall: {
    enabled: boolean
    exe_name: string
    remove_install_dir: boolean
    extra_remove_paths: string[]
  }
}

export interface FileEntry {
  source: string
  destination: string
  recursive: boolean
  filter: string | null
  overwrite: boolean
}

export type InstallStep =
  | { kind: 'extract'; label: string }
  | { kind: 'shortcut'; label: string; target: string; locations: string[]; name: string | null }
  | { kind: 'registry'; label: string; hive: string; key: string; value_name: string; value_data: string; value_type: string }
  | { kind: 'command'; label: string; command: string; args: string[]; wait: boolean; fail_on_error: boolean }
  | { kind: 'register_uninstaller'; label: string }

const DEFAULT_PROJECT: LynxProject = {
  app: {
    name: 'My App',
    version: '1.0.0',
    publisher: '',
    id: 'com.example.myapp',
    description: '',
    icon: null,
    default_install_dir: '{program_files}/{app_name}',
    url: null,
  },
  theme: {
    name: 'lynx-default',
    path: 'themes/lynx-default',
    accent_color: '#FF6B35',
    background_color: '#1A1A2E',
    custom_vars: {},
  },
  files: [],
  steps: [
    { kind: 'extract', label: 'Installing application files...' },
    { kind: 'shortcut', label: 'Creating shortcuts...', target: '{install_dir}/{app_name}.exe', locations: ['desktop', 'start_menu'], name: null },
    { kind: 'register_uninstaller', label: 'Finalizing installation...' },
  ],
  prerequisites: [],
  uninstall: {
    enabled: true,
    exe_name: 'uninstall.exe',
    remove_install_dir: true,
    extra_remove_paths: [],
  },
}

export default function App() {
  const [project, setProject] = useState<LynxProject>(DEFAULT_PROJECT)
  const [section, setSection] = useState<Section>('app')
  const [projectPath, setProjectPath] = useState<string | null>(null)
  const [isDirty, setIsDirty] = useState(false)

  const updateProject = useCallback((updater: (p: LynxProject) => LynxProject) => {
    setProject(prev => updater(prev))
    setIsDirty(true)
  }, [])

  const handleNew = useCallback(async () => {
    const fresh = await invoke<LynxProject>('new_project')
    setProject(fresh)
    setProjectPath(null)
    setIsDirty(false)
  }, [])

  const handleOpen = useCallback(async () => {
    const path = await pickLynxFile()
    if (!path) return
    try {
      const loaded = await invoke<LynxProject>('load_project', { path })
      setProject(loaded)
      setProjectPath(path)
      setIsDirty(false)
    } catch (e) {
      alert(`Failed to open project: ${e}`)
    }
  }, [])

  const handleSave = useCallback(async (): Promise<string | null> => {
    let path = projectPath
    if (!path) {
      path = await saveLynxFile(`${project.app.name.replace(/\s+/g, '-')}.lynx`)
      if (!path) return null
      setProjectPath(path)
    }
    try {
      await invoke('save_project', { path, projectJson: project })
      setIsDirty(false)
      return path
    } catch (e) {
      alert(`Failed to save project: ${e}`)
      return null
    }
  }, [project, projectPath])

  const handleSaveAs = useCallback(async () => {
    const path = await saveLynxFile(`${project.app.name.replace(/\s+/g, '-')}.lynx`)
    if (!path) return
    try {
      await invoke('save_project', { path, projectJson: project })
      setProjectPath(path)
      setIsDirty(false)
    } catch (e) {
      alert(`Failed to save project: ${e}`)
    }
  }, [project])

  return (
    <div className="app">
      <Titlebar
        projectName={project.app.name}
        projectPath={projectPath}
        isDirty={isDirty}
        onNew={handleNew}
        onOpen={handleOpen}
        onSave={handleSave}
        onSaveAs={handleSaveAs}
      />
      <div className="app-body">
        <Sidebar section={section} onSection={setSection} project={project} />
        <main className="main-content">
          {section === 'app'   && <AppSection   project={project} onChange={updateProject} />}
          {section === 'files' && <FilesSection project={project} onChange={updateProject} />}
          {section === 'steps' && <StepsSection project={project} onChange={updateProject} />}
          {section === 'theme' && <ThemeSection project={project} onChange={updateProject} />}
          {section === 'build' && <BuildSection project={project} projectPath={projectPath} onSave={handleSave} />}
        </main>
      </div>
    </div>
  )
}