import React from 'react';
import { useAuth } from '@/lib/auth';

export default function Settings() {
  const { user } = useAuth();

  return (
    <div className="min-h-screen bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-3xl mx-auto">
        <div className="bg-white shadow rounded-lg">
          <div className="px-6 py-4 border-b border-gray-200">
            <h1 className="text-2xl font-bold text-gray-900">Settings</h1>
            <p className="text-sm text-gray-500">Manage your account and application preferences</p>
          </div>

          <div className="p-6 space-y-6">
            <section>
              <h2 className="text-lg font-medium text-gray-900 mb-2">Account</h2>
              <p className="text-sm text-gray-600">Signed in as <span className="font-medium">{user?.email}</span></p>
            </section>

            <section className="border-t border-gray-200 pt-6">
              <h2 className="text-lg font-medium text-gray-900 mb-2">Preferences</h2>
              <p className="text-sm text-gray-600">More preferences coming soon.</p>
            </section>
          </div>
        </div>
      </div>
    </div>
  );
}
