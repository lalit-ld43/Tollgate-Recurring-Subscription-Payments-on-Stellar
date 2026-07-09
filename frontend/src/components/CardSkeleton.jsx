export default function CardSkeleton() {
  return (
    <div className="bg-asphalt-soft border border-asphalt-line rounded-lg p-4 sm:p-5 animate-pulse">
      <div className="h-3 w-24 bg-asphalt-line rounded mb-3" />
      <div className="h-5 w-32 bg-asphalt-line rounded mb-4" />
      <div className="h-16 bg-asphalt-line/60 rounded" />
    </div>
  )
}
