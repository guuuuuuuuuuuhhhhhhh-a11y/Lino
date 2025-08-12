import React, { useEffect, useMemo, useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Distro, DistroVersion } from './types'
import { DistroCard } from './components/DistroCard'
import { TopBar } from './components/TopBar'

export function App() {
  const [distros, setDistros] = useState<Distro[]>([])
  const [query, setQuery] = useState('')
  const [selected, setSelected] = useState<Distro | null>(null)
  const [versions, setVersions] = useState<DistroVersion[]>([])
  const [progress, setProgress] = useState<{phase:string, downloaded:number, total?:number, label:string}|null>(null)

  useEffect(() => {
    (async () => {
      const list = await invoke<Distro[]>('cmd_list_distros', {})
      setDistros(list)
    })()
    const un = listen('install-progress', (e: any) => {
      const { phase, downloaded, total, distro_id, version } = e.payload
      const label = `${distro_id} ${version}: ${phase}`
      setProgress({ phase, downloaded, total, label })
    })
    return () => { un.then(f => f()) }
  }, [])

  useEffect(() => {
    (async () => {
      if (!selected) { setVersions([]); return }
      const v = await invoke<DistroVersion[]>('cmd_versions', { distroId: selected.id })
      setVersions(v)
    })()
  }, [selected])

  const filtered = useMemo(() =>
    distros.filter(d => d.name.toLowerCase().includes(query.toLowerCase()) || d.id.includes(query.toLowerCase())),
  [distros, query])

  return (
    <div className="app">
      <TopBar query={query} onQueryChange={setQuery} onSettings={() => invoke('cmd_open_settings')} />
      {progress && (
        <div className="banner">
          <div>{progress.label}</div>
          {progress.total && (
            <div className="bar"><div className="fill" style={{ width: `${Math.min(100, (progress.downloaded / progress.total) * 100)}%` }} /></div>
          )}
        </div>
      )}
      <div className="grid">
        <AnimatePresence mode="popLayout">
          {filtered.map(d => (
            <motion.div key={d.id} layout initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -10 }}>
              <DistroCard distro={d} onSelect={() => setSelected(d)} onInstall={async (version, backend) => {
                await invoke('cmd_install', { req: { distroId: d.id, version, backend } as any })
              }} versions={versions} selected={selected?.id === d.id} />
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
    </div>
  )
}