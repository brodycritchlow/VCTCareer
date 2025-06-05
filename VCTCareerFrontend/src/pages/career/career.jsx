import React, { useState } from 'react';

const fakeInbox = [
  {
    id: 1,
    subject: 'Welcome to the League!',
    from: 'Commissioner',
    preview: 'Congratulations on joining the Valorant Pro League...',
    content: 'Congratulations on joining the Valorant Pro League! We are excited to have you. Your journey begins now. Good luck!'
  },
  {
    id: 2,
    subject: 'Contract Offer: Team Phoenix',
    from: 'Team Phoenix',
    preview: 'We would like to offer you a spot on our roster...',
    content: 'Team Phoenix is offering you a contract for 2 seasons with a competitive salary. Reply to negotiate or accept.'
  },
  {
    id: 3,
    subject: 'Scrim Schedule',
    from: 'Coach',
    preview: 'Please see the attached scrim schedule for next week...',
    content: 'Your scrim schedule for next week is attached. Be prepared and on time!'
  },
];

const Career = () => {
  const [openEmail, setOpenEmail] = useState(null);

  return (
    <div style={{
      display: 'flex',
      height: 'calc(100vh - 64px)',
      background: '#1a202c',
      padding: 32,
      gap: 32,
      position: 'relative',
      transition: 'all 0.4s cubic-bezier(0.4,0,0.2,1)',
    }}>
      {/* Left: Inbox */}
      <div
        style={{
          flex: openEmail ? 1.2 : 2,
          background: '#232b3a',
          borderRadius: 16,
          padding: 24,
          color: '#fff',
          boxShadow: '0 4px 24px rgba(0,0,0,0.12)',
          display: 'flex',
          flexDirection: 'column',
          minWidth: 0,
          transition: 'flex 0.4s cubic-bezier(0.4,0,0.2,1)',
          zIndex: 2,
        }}
      >
        <h2 style={{ fontSize: 22, fontWeight: 700, marginBottom: 16, letterSpacing: 0.5 }}>Inbox</h2>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          {fakeInbox.map(email => (
            <div
              key={email.id}
              onClick={() => setOpenEmail(email)}
              style={{
                background: '#283046',
                borderRadius: 10,
                padding: '14px 18px',
                cursor: 'pointer',
                boxShadow: '0 2px 8px rgba(0,0,0,0.08)',
                transition: 'background 0.2s',
                borderLeft: '4px solid #6366f1',
                opacity: openEmail ? 0.6 : 1,
                pointerEvents: openEmail ? 'none' : 'auto',
              }}
            >
              <div style={{ fontWeight: 600, fontSize: 16 }}>{email.subject}</div>
              <div style={{ color: '#a3a3a3', fontSize: 13, marginTop: 2 }}>{email.from}</div>
              <div style={{ color: '#cbd5e1', fontSize: 14, marginTop: 4 }}>{email.preview}</div>
            </div>
          ))}
        </div>
      </div>
      {/* Right: 2 stacked blocks */}
      <div
        style={{
          flex: openEmail ? 1.2 : 3,
          display: 'flex',
          flexDirection: 'column',
          gap: 32,
          minWidth: 0,
          transition: 'flex 0.4s cubic-bezier(0.4,0,0.2,1)',
          zIndex: 2,
        }}
      >
        {/* Top: Current Contract */}
        <div style={{
          flex: 0.8,
          background: '#232b3a',
          borderRadius: 16,
          padding: 24,
          color: '#fff',
          boxShadow: '0 4px 24px rgba(0,0,0,0.12)',
          display: 'flex',
          flexDirection: 'column',
          minHeight: 0,
          transition: 'all 0.4s cubic-bezier(0.4,0,0.2,1)',
        }}>
          <h2 style={{ fontSize: 20, fontWeight: 700, marginBottom: 12, letterSpacing: 0.5 }}>Current Contract</h2>
          <div style={{ display: 'flex', alignItems: 'center', gap: 18, marginBottom: 18 }}>
            <div style={{ width: 64, height: 64, borderRadius: 14, background: '#6366f1', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 36, fontWeight: 700, color: '#fff' }}>
              🦅
            </div>
            <div>
              <div style={{ fontWeight: 700, fontSize: 20, marginBottom: 2 }}>Team Phoenix</div>
              <div style={{ color: '#a3e635', fontWeight: 600, fontSize: 16, marginBottom: 2 }}>Salary: $120,000 / season</div>
              <div style={{ color: '#cbd5e1', fontSize: 15, marginBottom: 2 }}>Remaining Seasons: 2</div>
              <div style={{ color: '#cbd5e1', fontSize: 15, marginBottom: 2 }}>Role: Duelist</div>
              <div style={{ color: '#cbd5e1', fontSize: 15, marginBottom: 2 }}>Region: NA</div>
              <div style={{ color: '#cbd5e1', fontSize: 15, marginBottom: 2 }}>Signed: 2025-2027</div>
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 10, marginLeft: 28 }}>
              <button style={{
                background: '#334155', color: '#fff', border: 'none', borderRadius: 8, padding: '8px 16px', fontWeight: 600, fontSize: 14, cursor: 'pointer', transition: 'background 0.2s',
              }}>Request to be Benched</button>
              <button style={{
                background: '#f87171', color: '#fff', border: 'none', borderRadius: 8, padding: '8px 16px', fontWeight: 600, fontSize: 14, cursor: 'pointer', transition: 'background 0.2s',
              }}>Request to be Cut</button>
              <button style={{
                background: '#a3e635', color: '#232b3a', border: 'none', borderRadius: 8, padding: '8px 16px', fontWeight: 600, fontSize: 14, cursor: 'pointer', transition: 'background 0.2s',
              }}>Request More Money</button>
            </div>
          </div>
          <div style={{ color: '#a3a3a3', fontSize: 13 }}>
            (Replace this block with real contract data)
          </div>
        </div>
        {/* Bottom: Stats */}
        <div style={{
          flex: 1.6,
          background: '#232b3a',
          borderRadius: 16,
          padding: 24,
          color: '#fff',
          boxShadow: '0 4px 24px rgba(0,0,0,0.12)',
          display: 'flex',
          flexDirection: 'column',
          minHeight: 0,
          transition: 'all 0.4s cubic-bezier(0.4,0,0.2,1)',
          overflow: 'hidden',
        }}>
          <h2 style={{ fontSize: 20, fontWeight: 700, marginBottom: 18, letterSpacing: 0.5 }}>Stats</h2>
          {/* Stat bars */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 18, marginBottom: 18, paddingBottom: 24 }}>
            {[
              { label: 'HS%', value: 78, color: '#a3e635' },
              { label: 'Aim', value: 85, color: '#6366f1' },
              { label: 'IGL', value: 67, color: '#f472b6' },
              { label: 'Movement', value: 90, color: '#38bdf8' },
              { label: 'Game Sense', value: 72, color: '#fbbf24' },
              { label: 'Clutch', value: 61, color: '#f87171' },
              { label: 'Entry', value: 80, color: '#34d399' },
              { label: 'Support', value: 74, color: '#facc15' },
              { label: 'Utility', value: 69, color: '#818cf8' },
            ].map(stat => (
              <div key={stat.label} style={{ width: '100%' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
                  <span style={{ fontWeight: 600, fontSize: 15 }}>{stat.label}</span>
                  <span style={{ fontWeight: 700, fontSize: 15, color: stat.color }}>{stat.value}</span>
                </div>
                <div style={{
                  width: '100%',
                  height: 14,
                  background: '#283046',
                  borderRadius: 7,
                  overflow: 'hidden',
                }}>
                  <div style={{
                    width: `${stat.value}%`,
                    height: '100%',
                    background: stat.color,
                    borderRadius: 7,
                    transition: 'width 0.6s cubic-bezier(0.4,0,0.2,1)',
                  }} />
                </div>
              </div>
            ))}
          </div>
          <div style={{ color: '#a3a3a3', fontSize: 13, marginBottom: 8 }}>
            Recent Match: 23/14/6 (K/D/A)
          </div>
          <div style={{ color: '#a3a3a3', fontSize: 13, marginBottom: 8 }}>
            Win Rate: 62% | Avg. ACS: 245 | KAST: 74%
          </div>
          <div style={{ color: '#a3a3a3', fontSize: 13 }}>
            (Replace this block with real stats data)
          </div>
        </div>
      </div>
      {/* Email open view - slides in and overlays, but right blocks shrink */}
      {openEmail && (
        <div
          style={{
            position: 'absolute',
            top: 32,
            left: `calc(33.333% + 48px)`, // aligns with the gap between left and right blocks
            width: '33.333%', // fits exactly in the gap
            height: 'calc(100% - 64px)',
            background: '#232b3a',
            borderRadius: 16,
            padding: 32,
            color: '#fff',
            boxShadow: '0 4px 24px rgba(0,0,0,0.18)',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'flex-start',
            justifyContent: 'center',
            animation: 'fadeInEmail 0.4s',
            zIndex: 10,
            transition: 'all 0.4s cubic-bezier(0.4,0,0.2,1)',
          }}
        >
          <button
            onClick={() => setOpenEmail(null)}
            style={{
              alignSelf: 'flex-end',
              background: '#6366f1',
              color: '#fff',
              border: 'none',
              borderRadius: 8,
              padding: '8px 18px',
              fontWeight: 600,
              fontSize: 15,
              marginBottom: 18,
              cursor: 'pointer',
            }}
          >
            Close
          </button>
          <div style={{ fontWeight: 700, fontSize: 22, marginBottom: 8 }}>{openEmail.subject}</div>
          <div style={{ color: '#a3e635', fontWeight: 500, fontSize: 15, marginBottom: 6 }}>From: {openEmail.from}</div>
          <div style={{ color: '#cbd5e1', fontSize: 16, marginBottom: 18 }}>{openEmail.content}</div>
          <div style={{ color: '#a3a3a3', fontSize: 13 }}>(Replace this block with real message data)</div>
        </div>
      )}
      <style>{`
        @keyframes fadeInEmail {
          from { opacity: 0; transform: translateY(32px) scale(0.98); }
          to { opacity: 1; transform: translateY(0) scale(1); }
        }
      `}</style>
    </div>
  );
};

export default Career;