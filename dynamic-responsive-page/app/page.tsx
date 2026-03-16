"use client"

import { useState } from "react"
import ServiceAnnouncement from "@/components/service-announcement"

export default function Home() {
  const [currentName, setCurrentName] = useState("Pásztor Balázs")

  // This function would be used if you want to toggle between names
  const toggleName = () => {
    setCurrentName((prev) => (prev === "Pásztor Balázs" ? "Hajduné Csernák Erzsébet" : "Pásztor Balázs"))
  }

  return (
    <main className="min-h-screen flex items-center justify-center">
      <ServiceAnnouncement name={currentName} />
    </main>
  )
}
