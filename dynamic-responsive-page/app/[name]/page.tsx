import ServiceAnnouncement from "@/components/service-announcement"

export default function NamePage({ params }: { params: { name: string } }) {
  // Decode URL-encoded name and replace hyphens with spaces
  const decodedName = decodeURIComponent(params.name).replace(/-/g, " ")

  return (
    <main className="min-h-screen flex items-center justify-center">
      <ServiceAnnouncement name={decodedName} />
    </main>
  )
}
