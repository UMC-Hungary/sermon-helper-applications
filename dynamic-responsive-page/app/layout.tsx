import type React from "react"
import { Oswald } from "next/font/google"
import "./globals.css"
import type { Metadata } from "next"

const oswald = Oswald({
  subsets: ["latin", "latin-ext"],
  weight: ["400", "700"],
  variable: "--font-oswald",
})

export const metadata: Metadata = {
  title: "Nyíregyházi Metodista Gyülekezet",
  description: "Vasárnapi Istentisztelet",
    generator: 'v0.app'
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="hu">
      <body className={`${oswald.variable} font-oswald`}>{children}</body>
    </html>
  )
}
