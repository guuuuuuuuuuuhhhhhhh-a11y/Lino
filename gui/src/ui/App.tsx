import React, { useEffect, useMemo, useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'
import { Distro, DistroVersion } from './types'
import { DistroCard } from './components/DistroCard'
import { TopBar } from './components/TopBar'

export function App() {
  const [distros, setDistros] = useState<Distro[]>([])
  const [query, setQuery] = useState('')
  const [selected, setSelected] = useState<Distro | null>(null)
  const [versions, setVersions] = useState<DistroVersion[]>([])

  useEffect(() => {
    (async () => {
      const list = await invoke<Distro[]>('cmd_list_distros', {})
      setDistros(list)
    })()
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
      <div className="grid">
        <AnimatePresence mode="popLayout">
          {filtered.map(d => (
            <motion.div key={d.id} layout initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -10 }}>
              <DistroCard distro={d} onSelect={() => setSelected(d)} onInstall={async (version, backend) => {
                await invoke('cmd_install', { distroId: d.id, version, backend })
              }} versions={versions} selected={selected?.id === d.id} />
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
    </div>
  )
}