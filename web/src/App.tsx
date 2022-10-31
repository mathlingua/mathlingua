import React from 'react';
import { Shell } from './Shell';
import { Sidebar } from './Sidebar';

export function App() {
  return (
    <Shell
      topbarContent={<></>}
      sidebarContent={<Sidebar />}
      mainContent={<></>} />
  );
}
