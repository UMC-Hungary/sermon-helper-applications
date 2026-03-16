import Image from "next/image"

interface ServiceAnnouncementProps {
  name: string
}

export default function ServiceAnnouncement({ name }: ServiceAnnouncementProps) {
  return (
    <div className="aspect-container">
      <div className="content">
        <h1 className="name-title">{name}</h1>
        <div className="service-info">
          <span>VASÁRNAPI ISTENTISZTELET</span>
          <span className="dot"></span>
          <span>IGEHIRDETÉS</span>
        </div>
      </div>

      <div className="logo-container">
        <Image src="/logo.svg" alt="Nyíregyházi Metodista Gyülekezet" width={400} height={120} priority />
      </div>
    </div>
  )
}
