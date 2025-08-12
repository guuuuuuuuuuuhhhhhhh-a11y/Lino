import React from 'react'

export function TopBar({ query, onQueryChange, onSettings }: { query: string, onQueryChange: (q: string) => void, onSettings: () => void }) {
  return (
    <div className="topbar">
      <div className="brand">Linux Distro Manager</div>
      <input placeholder="Search distributions" value={query} onChange={e => onQueryChange(e.target.value)} />
      <button onClick={onSettings}>Settings</button>
    </div>
  )
}