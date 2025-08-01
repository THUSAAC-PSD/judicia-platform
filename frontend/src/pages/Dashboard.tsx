import { Layout } from '../components/Layout'
import { Code, Trophy, Users, TrendingUp } from 'lucide-react'

export function Dashboard() {
  const stats = [
    { name: 'Total Problems', value: '156', icon: Code, color: 'bg-blue-500' },
    { name: 'Active Contests', value: '3', icon: Trophy, color: 'bg-green-500' },
    { name: 'Registered Users', value: '2,847', icon: Users, color: 'bg-purple-500' },
    { name: 'Submissions Today', value: '421', icon: TrendingUp, color: 'bg-orange-500' },
  ]

  const recentSubmissions = [
    { id: 1, problem: 'Two Sum', status: 'Accepted', time: '2 minutes ago' },
    { id: 2, problem: 'Binary Search', status: 'Wrong Answer', time: '5 minutes ago' },
    { id: 3, problem: 'DFS Traversal', status: 'Accepted', time: '10 minutes ago' },
  ]

  const activeContests = [
    { id: 1, name: 'Weekly Contest #45', type: 'IOI', participants: 234, timeLeft: '2h 45m' },
    { id: 2, name: 'ICPC Regional Practice', type: 'ICPC', participants: 156, timeLeft: '1h 20m' },
  ]

  return (
    <Layout>
      <div className="p-6">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Dashboard</h1>
          <p className="text-gray-600">Welcome back! Here's what's happening.</p>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {stats.map((stat) => (
            <div key={stat.name} className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
              <div className="flex items-center">
                <div className={`${stat.color} rounded-lg p-3`}>
                  <stat.icon className="h-6 w-6 text-white" />
                </div>
                <div className="ml-4">
                  <p className="text-sm font-medium text-gray-600">{stat.name}</p>
                  <p className="text-2xl font-bold text-gray-900">{stat.value}</p>
                </div>
              </div>
            </div>
          ))}
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Recent Submissions */}
          <div className="bg-white rounded-lg shadow-sm border border-gray-200">
            <div className="p-6 border-b border-gray-200">
              <h2 className="text-lg font-semibold text-gray-900">Recent Submissions</h2>
            </div>
            <div className="p-6">
              <div className="space-y-4">
                {recentSubmissions.map((submission) => (
                  <div key={submission.id} className="flex items-center justify-between">
                    <div>
                      <p className="font-medium text-gray-900">{submission.problem}</p>
                      <p className="text-sm text-gray-500">{submission.time}</p>
                    </div>
                    <span className={`px-2 py-1 text-xs font-medium rounded-full ${
                      submission.status === 'Accepted' 
                        ? 'bg-green-100 text-green-800' 
                        : 'bg-red-100 text-red-800'
                    }`}>
                      {submission.status}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Active Contests */}
          <div className="bg-white rounded-lg shadow-sm border border-gray-200">
            <div className="p-6 border-b border-gray-200">
              <h2 className="text-lg font-semibold text-gray-900">Active Contests</h2>
            </div>
            <div className="p-6">
              <div className="space-y-4">
                {activeContests.map((contest) => (
                  <div key={contest.id} className="border border-gray-200 rounded-lg p-4">
                    <div className="flex items-center justify-between mb-2">
                      <h3 className="font-medium text-gray-900">{contest.name}</h3>
                      <span className="px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800 rounded-full">
                        {contest.type}
                      </span>
                    </div>
                    <div className="flex items-center justify-between text-sm text-gray-500">
                      <span>{contest.participants} participants</span>
                      <span>Ends in {contest.timeLeft}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  )
}
