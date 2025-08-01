import { useState } from 'react'
import { Layout } from '../components/Layout'
import { Calendar, Users, Trophy, Clock } from 'lucide-react'
import { formatDate } from '../lib/utils'
import type { Contest } from '../types'

export function Contests() {
  const [activeTab, setActiveTab] = useState<'upcoming' | 'running' | 'past'>('running')

  // Mock data - replace with API call
  const contests: Contest[] = [
    {
      id: '1',
      name: 'Weekly Contest #45',
      description: 'Practice contest featuring algorithmic problems of varying difficulty',
      contestType: 'IOI',
      startTime: '2024-08-01T10:00:00Z',
      endTime: '2024-08-01T13:00:00Z',
      problems: [],
      participants: [],
      status: 'Running',
      isPublic: true
    },
    {
      id: '2',
      name: 'ICPC Regional Practice',
      description: 'Prepare for ICPC regionals with team-based problem solving',
      contestType: 'ICPC',
      startTime: '2024-08-01T14:00:00Z',
      endTime: '2024-08-01T19:00:00Z',
      problems: [],
      participants: [],
      status: 'Scheduled',
      isPublic: true
    },
    {
      id: '3',
      name: 'Advanced Algorithms Challenge',
      description: 'High-level algorithmic problems for experienced programmers',
      contestType: 'Custom',
      startTime: '2024-07-28T10:00:00Z',
      endTime: '2024-07-28T15:00:00Z',
      problems: [],
      participants: [],
      status: 'Finished',
      isPublic: true
    }
  ]

  const filteredContests = contests.filter(contest => {
    switch (activeTab) {
      case 'upcoming':
        return contest.status === 'Scheduled'
      case 'running':
        return contest.status === 'Running'
      case 'past':
        return contest.status === 'Finished'
      default:
        return false
    }
  })

  const getContestTypeColor = (type: string) => {
    switch (type) {
      case 'IOI':
        return 'bg-blue-100 text-blue-800'
      case 'ICPC':
        return 'bg-green-100 text-green-800'
      case 'AtCoder':
        return 'bg-purple-100 text-purple-800'
      default:
        return 'bg-gray-100 text-gray-800'
    }
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Running':
        return 'bg-green-100 text-green-800'
      case 'Scheduled':
        return 'bg-yellow-100 text-yellow-800'
      case 'Finished':
        return 'bg-gray-100 text-gray-800'
      default:
        return 'bg-gray-100 text-gray-800'
    }
  }

  const calculateDuration = (start: string, end: string) => {
    const startTime = new Date(start)
    const endTime = new Date(end)
    const durationMs = endTime.getTime() - startTime.getTime()
    const hours = Math.floor(durationMs / (1000 * 60 * 60))
    const minutes = Math.floor((durationMs % (1000 * 60 * 60)) / (1000 * 60))
    return `${hours}h ${minutes}m`
  }

  return (
    <Layout>
      <div className="p-6">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Contests</h1>
          <p className="text-gray-600">Participate in programming contests and challenges.</p>
        </div>

        {/* Tabs */}
        <div className="mb-6">
          <div className="border-b border-gray-200">
            <nav className="-mb-px flex space-x-8">
              {[
                { key: 'running', label: 'Running', count: contests.filter(c => c.status === 'Running').length },
                { key: 'upcoming', label: 'Upcoming', count: contests.filter(c => c.status === 'Scheduled').length },
                { key: 'past', label: 'Past', count: contests.filter(c => c.status === 'Finished').length }
              ].map(tab => (
                <button
                  key={tab.key}
                  onClick={() => setActiveTab(tab.key as 'upcoming' | 'running' | 'past')}
                  className={`py-2 px-1 border-b-2 font-medium text-sm ${
                    activeTab === tab.key
                      ? 'border-primary-500 text-primary-600'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  }`}
                >
                  {tab.label} ({tab.count})
                </button>
              ))}
            </nav>
          </div>
        </div>

        {/* Contests Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredContests.map((contest) => (
            <div key={contest.id} className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 hover:shadow-md transition-shadow cursor-pointer">
              <div className="flex items-start justify-between mb-4">
                <div className="flex-1">
                  <h3 className="text-lg font-semibold text-gray-900 mb-2">{contest.name}</h3>
                  <p className="text-gray-600 text-sm line-clamp-2">{contest.description}</p>
                </div>
                <div className="flex flex-col items-end space-y-2">
                  <span className={`px-2 py-1 text-xs font-medium rounded-full ${getContestTypeColor(contest.contestType)}`}>
                    {contest.contestType}
                  </span>
                  <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(contest.status)}`}>
                    {contest.status}
                  </span>
                </div>
              </div>

              <div className="space-y-3">
                <div className="flex items-center text-sm text-gray-600">
                  <Calendar className="h-4 w-4 mr-2" />
                  <span>{formatDate(contest.startTime)}</span>
                </div>

                <div className="flex items-center text-sm text-gray-600">
                  <Clock className="h-4 w-4 mr-2" />
                  <span>{calculateDuration(contest.startTime, contest.endTime)}</span>
                </div>

                <div className="flex items-center text-sm text-gray-600">
                  <Users className="h-4 w-4 mr-2" />
                  <span>{contest.participants.length} participants</span>
                </div>

                <div className="flex items-center text-sm text-gray-600">
                  <Trophy className="h-4 w-4 mr-2" />
                  <span>{contest.problems.length} problems</span>
                </div>
              </div>

              <div className="mt-6 pt-4 border-t border-gray-200">
                {contest.status === 'Running' && (
                  <button className="w-full bg-primary-500 text-white py-2 px-4 rounded-lg hover:bg-primary-600 transition-colors">
                    Join Contest
                  </button>
                )}
                {contest.status === 'Scheduled' && (
                  <button className="w-full bg-yellow-500 text-white py-2 px-4 rounded-lg hover:bg-yellow-600 transition-colors">
                    Register
                  </button>
                )}
                {contest.status === 'Finished' && (
                  <button className="w-full bg-gray-500 text-white py-2 px-4 rounded-lg hover:bg-gray-600 transition-colors">
                    View Results
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>

        {filteredContests.length === 0 && (
          <div className="text-center py-12">
            <Trophy className="h-12 w-12 text-gray-400 mx-auto mb-4" />
            <p className="text-gray-500">No {activeTab} contests found.</p>
          </div>
        )}
      </div>
    </Layout>
  )
}
