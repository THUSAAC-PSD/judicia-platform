import React, { useState } from 'react';
import { Link } from 'wouter';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Plus, Edit, Trash2, Calendar, Clock, Users, UserPlus, UserMinus, User } from 'lucide-react';
import { apiRequest } from '../lib/queryClient';
import BulkUserForm from '../components/BulkUserForm';
import CreateUserForm from '../components/CreateUserForm';
import type { Contest } from '@shared/schema';

interface ContestFormData {
  title: string;
  description: string;
  startTime: string;
  duration: number;
}

interface User {
  id: string;
  username: string;
  email: string;
  roles: string[];
  firstName?: string;
  lastName?: string;
}

interface ContestAdmin {
  id: string;
  username: string;
  email: string;
  roles: string[];
  firstName?: string;
  lastName?: string;
}

export default function AdminPanel() {
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [showBulkUserForm, setShowBulkUserForm] = useState(false);
  const [showCreateUserForm, setShowCreateUserForm] = useState(false);
  const [showContestAdminSection, setShowContestAdminSection] = useState(false);
  const [selectedContestForAdmin, setSelectedContestForAdmin] = useState<string>('');
  const [formData, setFormData] = useState<ContestFormData>({
    title: '',
    description: '',
    startTime: '',
    duration: 7200 // 2 hours default
  });

  const queryClient = useQueryClient();

  const { data: contests = [], isLoading } = useQuery<Contest[]>({
    queryKey: ['/api/contests']
  });

  const { data: users = [], isLoading: usersLoading } = useQuery<User[]>({
    queryKey: ['/api/users']
  });

  // Get all users with contest_admin role
  const contestAdminUsers = users.filter((user) => user.roles.includes('contest_admin'));

  // Get contest admins for selected contest
  const { data: contestAdmins = [], isLoading: contestAdminsLoading } = useQuery<ContestAdmin[]>({
    queryKey: ['/api/contests', selectedContestForAdmin, 'admins'],
    enabled: !!selectedContestForAdmin,
  });

  const createMutation = useMutation({
    mutationFn: async (data: ContestFormData) => {
      const start = new Date(data.startTime);
      const end = new Date(start.getTime() + data.duration * 1000);
      const payload = {
        title: data.title,
        description: data.description,
        startTime: start.toISOString(),
        endTime: end.toISOString(),
        difficulty: 'intermediate',
        maxParticipants: null as number | null,
      };
      return apiRequest('/api/contests', {
        method: 'POST',
        body: JSON.stringify(payload)
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['/api/contests'] });
      setShowCreateForm(false);
      setFormData({ title: '', description: '', startTime: '', duration: 7200 });
    }
  });

  const deleteMutation = useMutation({
    mutationFn: async (contestId: string) => {
      return apiRequest(`/api/contests/${contestId}`, {
        method: 'DELETE'
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['/api/contests'] });
    }
  });

  const assignContestAdminMutation = useMutation({
    mutationFn: async ({ contestId, userId }: { contestId: string; userId: string }) => {
      return apiRequest(`/api/contests/${contestId}/admins`, {
        method: 'POST',
        body: JSON.stringify({ userId })
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['/api/contests', selectedContestForAdmin, 'admins'] });
    }
  });

  const removeContestAdminMutation = useMutation({
    mutationFn: async ({ contestId, userId }: { contestId: string; userId: string }) => {
      return apiRequest(`/api/contests/${contestId}/admins/${userId}`, {
        method: 'DELETE'
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['/api/contests', selectedContestForAdmin, 'admins'] });
    }
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    createMutation.mutate(formData);
  };

  const formatDuration = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  };

  if (isLoading) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 rounded w-1/3 mb-8"></div>
          <div className="space-y-4">
            {[...Array(5)].map((_, i) => (
              <div key={i} className="h-20 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Admin Panel</h1>
          <p className="mt-2 text-gray-600">Manage contests and platform settings</p>
        </div>
        <Link
          href="/contests"
          className="px-4 py-2 text-sm text-gray-600 hover:text-gray-800 border border-gray-300 rounded-md hover:bg-gray-50"
        >
          ← Back to contests
        </Link>
      </div>

      {/* Create Contest Section */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 mb-8">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">Create New Contest</h2>
        </div>
        
        <div className="p-6">
          {!showCreateForm ? (
            <button
              onClick={() => setShowCreateForm(true)}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              data-testid="button-create-contest"
            >
              <Plus className="h-4 w-4 mr-2" />
              Create Contest
            </button>
          ) : (
            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label htmlFor="title" className="block text-sm font-medium text-gray-700">
                  Contest Title
                </label>
                <input
                  id="title"
                  type="text"
                  value={formData.title}
                  onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                  placeholder="Enter contest title"
                  required
                  className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  data-testid="input-contest-title"
                />
              </div>

              <div>
                <label htmlFor="description" className="block text-sm font-medium text-gray-700">
                  Description
                </label>
                <textarea
                  id="description"
                  value={formData.description}
                  onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                  placeholder="Enter contest description"
                  rows={3}
                  required
                  className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  data-testid="textarea-contest-description"
                />
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label htmlFor="startTime" className="block text-sm font-medium text-gray-700">
                    Start Time
                  </label>
                  <input
                    id="startTime"
                    type="datetime-local"
                    value={formData.startTime}
                    onChange={(e) => setFormData({ ...formData, startTime: e.target.value })}
                    required
                    className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                    data-testid="input-contest-start-time"
                  />
                </div>

                <div>
                  <label htmlFor="duration" className="block text-sm font-medium text-gray-700">
                    Duration (seconds)
                  </label>
                  <input
                    id="duration"
                    type="number"
                    value={formData.duration}
                    onChange={(e) => setFormData({ ...formData, duration: parseInt(e.target.value) })}
                    min="600"
                    step="300"
                    required
                    className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                    data-testid="input-contest-duration"
                  />
                </div>
              </div>

              <div className="flex gap-3">
                <button
                  type="submit"
                  disabled={createMutation.isPending}
                  className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                  data-testid="button-submit-contest"
                >
                  {createMutation.isPending ? 'Creating...' : 'Create Contest'}
                </button>
                <button
                  type="button"
                  onClick={() => setShowCreateForm(false)}
                  className="px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                  data-testid="button-cancel-contest"
                >
                  Cancel
                </button>
              </div>
            </form>
          )}
        </div>
      </div>

      {/* Existing Contests */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">
            Existing Contests ({contests.length})
          </h2>
        </div>
        
        <div className="overflow-hidden">
          {contests.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-gray-500">No contests created yet.</p>
            </div>
          ) : (
            <div className="divide-y divide-gray-200">
              {contests.map((contest) => (
                <div key={contest.id} className="p-6">
                  <div className="flex items-center justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-4 mb-2">
                        <h3 className="text-lg font-medium text-gray-900">
                          {contest.title}
                        </h3>
                        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                          new Date() < new Date(contest.startTime) 
                            ? 'bg-blue-100 text-blue-800' 
                            : new Date() > new Date(contest.endTime)
                            ? 'bg-gray-100 text-gray-800'
                            : 'bg-green-100 text-green-800'
                        }`}>
                          {new Date() < new Date(contest.startTime) 
                            ? 'Upcoming' 
                            : new Date() > new Date(contest.endTime)
                            ? 'Ended'
                            : 'Running'}
                        </span>
                      </div>
                      
                      <p className="text-sm text-gray-600 mb-3">
                        {contest.description}
                      </p>
                      
                      <div className="flex items-center gap-6 text-sm text-gray-500">
                        <div className="flex items-center">
                          <Calendar className="h-4 w-4 mr-1" />
                          {new Date(contest.startTime).toLocaleDateString()}
                        </div>
                        <div className="flex items-center">
                          <Clock className="h-4 w-4 mr-1" />
                          {(() => {
                            const start = new Date(contest.startTime).getTime();
                            const end = new Date(contest.endTime).getTime();
                            const seconds = Math.max(0, Math.floor((end - start) / 1000));
                            return formatDuration(seconds);
                          })()}
                        </div>
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-2 ml-4">
                      <Link
                        href={`/contests/${contest.id}`}
                        className="p-2 text-gray-400 hover:text-gray-600"
                        data-testid={`button-view-${contest.id}`}
                      >
                        <Edit className="h-4 w-4" />
                      </Link>
                      <button
                        onClick={() => deleteMutation.mutate(contest.id)}
                        disabled={deleteMutation.isPending}
                        className="p-2 text-gray-400 hover:text-red-600 disabled:opacity-50"
                        data-testid={`button-delete-${contest.id}`}
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Contest Admin Management Section */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 mb-8">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">Contest Admin Management</h2>
        </div>
        
        <div className="p-6">
          {!showContestAdminSection ? (
            <button
              onClick={() => setShowContestAdminSection(true)}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
            >
              <UserPlus className="h-4 w-4 mr-2" />
              Manage Contest Admins
            </button>
          ) : (
            <div className="space-y-6">
              <div>
                <label htmlFor="contestSelect" className="block text-sm font-medium text-gray-700 mb-2">
                  Select Contest
                </label>
                <select
                  id="contestSelect"
                  value={selectedContestForAdmin}
                  onChange={(e) => setSelectedContestForAdmin(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-green-500"
                >
                  <option value="">Choose a contest...</option>
                  {contests.map((contest) => (
                    <option key={contest.id} value={contest.id}>
                      {contest.title}
                    </option>
                  ))}
                </select>
              </div>

              {selectedContestForAdmin && (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  {/* Available Contest Admins */}
                  <div>
                    <h3 className="text-md font-medium text-gray-900 mb-3">Available Contest Admins</h3>
                    <div className="space-y-2 max-h-60 overflow-y-auto">
                      {contestAdminUsers.length === 0 ? (
                        <p className="text-sm text-gray-500">No users with contest_admin role found.</p>
                      ) : (
                        contestAdminUsers
                          .filter(user => !contestAdmins.some(admin => admin.id === user.id))
                          .map((user) => (
                            <div key={user.id} className="flex items-center justify-between p-3 border border-gray-200 rounded-md">
                              <div>
                                <p className="text-sm font-medium text-gray-900">{user.username}</p>
                                <p className="text-xs text-gray-500">{user.email}</p>
                              </div>
                              <button
                                onClick={() => assignContestAdminMutation.mutate({ 
                                  contestId: selectedContestForAdmin, 
                                  userId: user.id 
                                })}
                                disabled={assignContestAdminMutation.isPending}
                                className="inline-flex items-center px-2 py-1 text-xs font-medium rounded text-green-700 bg-green-100 hover:bg-green-200 disabled:opacity-50"
                              >
                                <UserPlus className="h-3 w-3 mr-1" />
                                Assign
                              </button>
                            </div>
                          ))
                      )}
                    </div>
                  </div>

                  {/* Assigned Contest Admins */}
                  <div>
                    <h3 className="text-md font-medium text-gray-900 mb-3">Assigned Contest Admins</h3>
                    <div className="space-y-2 max-h-60 overflow-y-auto">
                      {contestAdminsLoading ? (
                        <p className="text-sm text-gray-500">Loading...</p>
                      ) : contestAdmins.length === 0 ? (
                        <p className="text-sm text-gray-500">No contest admins assigned.</p>
                      ) : (
                        contestAdmins.map((admin) => (
                          <div key={admin.id} className="flex items-center justify-between p-3 border border-gray-200 rounded-md">
                            <div>
                              <p className="text-sm font-medium text-gray-900">{admin.username}</p>
                              <p className="text-xs text-gray-500">{admin.email}</p>
                            </div>
                            <button
                              onClick={() => removeContestAdminMutation.mutate({ 
                                contestId: selectedContestForAdmin, 
                                userId: admin.id 
                              })}
                              disabled={removeContestAdminMutation.isPending}
                              className="inline-flex items-center px-2 py-1 text-xs font-medium rounded text-red-700 bg-red-100 hover:bg-red-200 disabled:opacity-50"
                            >
                              <UserMinus className="h-3 w-3 mr-1" />
                              Remove
                            </button>
                          </div>
                        ))
                      )}
                    </div>
                  </div>
                </div>
              )}

              <button
                onClick={() => {
                  setShowContestAdminSection(false);
                  setSelectedContestForAdmin('');
                }}
                className="px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
              >
                Close
              </button>
            </div>
          )}
        </div>
      </div>

      {/* User Management Section */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 mb-8">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">User Management</h2>
        </div>
        
        <div className="p-6">
          <div className="flex gap-3">
            <button
              onClick={() => setShowCreateUserForm(true)}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              <User className="h-4 w-4 mr-2" />
              Create User
            </button>
            <button
              onClick={() => setShowBulkUserForm(true)}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-purple-600 hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500"
            >
              <Users className="h-4 w-4 mr-2" />
              Bulk Create Contestants
            </button>
          </div>
          
          <div className="mt-4 text-sm text-gray-600">
            <p><strong>Create User:</strong> Create individual users with specific roles (contestant, contest_admin, admin)</p>
            <p><strong>Bulk Create:</strong> Quickly create multiple contestant users for competitions</p>
          </div>
        </div>
      </div>

      {showCreateUserForm && (
        <CreateUserForm
          onClose={() => setShowCreateUserForm(false)}
          onSuccess={() => {
            queryClient.invalidateQueries({ queryKey: ['/api/users'] });
          }}
        />
      )}

      {showBulkUserForm && (
        <BulkUserForm
          onClose={() => setShowBulkUserForm(false)}
          onSuccess={() => {
            queryClient.invalidateQueries({ queryKey: ['/api/users'] });
          }}
        />
      )}
    </div>
  );
}