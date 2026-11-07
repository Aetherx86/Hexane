// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

import React from 'react';
import './globals.css';

export const metadata = {
  title: 'Hexene Telemetry & Observability Engine',
  description: 'Distributed real-time log ingestion, indexing, and analytical query dashboard',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className="min-h-screen bg-[#0b0f19] text-gray-100 flex flex-col">
        {children}
      </body>
    </html>
  );
}
