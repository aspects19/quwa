import { createFileRoute } from '@tanstack/react-router'
import Header from '../components/Header'
import ChatInterface from '../components/ChatInterface'

export const Route = createFileRoute('/')({
  component: RouteComponent,
})


function RouteComponent() {
  return (
    <div className="min-h-screen flex flex-col">
      <Header />  
      <main className="flex-1 flex items-center justify-center p-4">
        <ChatInterface />
      </main>
    </div>
  )
}
