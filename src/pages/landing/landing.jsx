import React, { useRef, useEffect, useState } from 'react';

const milestones = [
    'Hit Radiant',
    'Amateur Team',
    'Open Qualifiers',
    'Challengers League (Tier 2)',
    'VCT Partner League (Tier 1)',
    'International Events (Masters/Champions)',
];

const FAKE_SCROLL_INTERVAL = 1200; // ms
const SCROLL_THROTTLE_MS = 250;

const Landing = () => {
    const itemRefs = useRef([]);
    const [activeIdx, setActiveIdx] = useState(0);
    const [fakeScroll, setFakeScroll] = useState(0);
    const lastScroll = useRef(0);
    const [canStart, setCanStart] = useState(false);

    useEffect(() => {
        const handleWheel = (e) => {
            // Only prevent scroll if not at the ends
            const now = Date.now();
            if (now - lastScroll.current < SCROLL_THROTTLE_MS) return;
            lastScroll.current = now;
            // Offset scroll immediately before any React state update
            window.scrollTo({ top: 0, left: 0, behavior: 'auto' });
            e.preventDefault();
            setActiveIdx(idx => {
                if (e.deltaY > 0) {
                    if (idx < milestones.length - 1) return idx + 1;
                    return idx;
                } else {
                    if (idx > 0) return idx - 1;
                    return idx;
                }
            });
            document.body.style.overflow = 'hidden';

        };
        // Attach wheel event to the roadmap container only
        const roadmap = document.getElementById('roadmap-scroll-lock');
        if (roadmap) {
            roadmap.addEventListener('wheel', handleWheel, { passive: false });
        }
        return () => {
            if (roadmap) {
                roadmap.removeEventListener('wheel', handleWheel);
            }
        };
    }, [activeIdx]);

    useEffect(() => {
        setCanStart(activeIdx === milestones.length - 1);
    }, [activeIdx]);

    return (
        <div
            id="roadmap-scroll-lock"
            style={{
                minHeight: '100vh',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexDirection: 'row',
            }}
        >
            {/* Roadmap on the left */}
            <div style={{
                flex: 1,
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'flex-end',
                justifyContent: 'center',
                height: '100%',
            }}>
                <div style={{ display: 'flex', flexDirection: 'row', alignItems: 'flex-start', position: 'relative' }}>
                    {/* Timeline (dots and lines) */}
                    <div style={{
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                        position: 'relative',
                        minWidth: 32,
                        paddingTop: 32,
                        paddingBottom: 8,
                    }}>
                        {milestones.map((_, idx) => (
                            <React.Fragment key={idx}>
                                <div style={{
                                    width: activeIdx === idx ? 22 : 16,
                                    height: activeIdx === idx ? 22 : 16,
                                    borderRadius: '50%',
                                    background: activeIdx === idx ? '#6366f1' : '#232b3a',
                                    border: activeIdx === idx ? '4px solid #6366f1' : '3px solid #2d3748',
                                    boxShadow: activeIdx === idx ? '0 0 0 4px #2d3748' : 'none',
                                    transition: 'all 0.3s cubic-bezier(.4,2,.6,1)',
                                    zIndex: 1,
                                    margin: 0,
                                    boxSizing: 'border-box',
                                }} />
                                {idx !== milestones.length - 1 && (
                                    <div style={{
                                        width: 4,
                                        height: 54 + 22, // 54 (label minHeight) + 22 (dot height) to connect dots
                                        marginTop: -11, // half of dot height to overlap
                                        background: '#2d3748',
                                        marginLeft: 'auto',
                                        marginRight: 'auto',
                                        transition: 'background 0.3s',
                                    }} />
                                )}
                            </React.Fragment>
                        ))}
                    </div>
                    {/* Milestone labels */}
                    <div style={{
                        display: 'flex',
                        flexDirection: 'column',
                        justifyContent: 'flex-start',
                        marginLeft: 24,
                        gap: 0,
                        paddingTop: 32,
                        paddingBottom: 8,
                    }}>
                        {milestones.map((milestone, idx) => (
                            <div
                                key={milestone}
                                ref={el => itemRefs.current[idx] = el}
                                style={{
                                    minHeight: 54,
                                    display: 'flex',
                                    alignItems: 'center',
                                    transition: 'all 0.3s cubic-bezier(.4,2,.6,1)',
                                    marginTop: idx === 0 ? 0 : 22, // match dot height for all but first
                                }}
                            >
                                <span style={{
                                    fontSize: activeIdx === idx ? '1.25rem' : '1.05rem',
                                    color: activeIdx === idx ? '#fff' : '#cbd5e1',
                                    background: activeIdx === idx ? '#232b3a' : '#232b3a',
                                    padding: activeIdx === idx ? '16px 28px' : '8px 18px',
                                    borderRadius: 8,
                                    border: activeIdx === idx ? '2px solid #6366f1' : '1px solid #2d3748',
                                    boxShadow: activeIdx === idx ? '0 4px 16px rgba(99,102,241,0.10)' : '0 2px 8px rgba(0,0,0,0.03)',
                                    fontWeight: activeIdx === idx ? 600 : 400,
                                    letterSpacing: activeIdx === idx ? '0.01em' : '0',
                                    transition: 'all 0.3s cubic-bezier(.4,2,.6,1)',
                                }}>{milestone}</span>
                            </div>
                        ))}
                    </div>
                </div>
            </div>
            {/* Vertical divider */}
            <div style={{
                width: 2,
                height: 360,
                background: '#232b3a',
                margin: '0 48px',
                borderRadius: 2,
                alignSelf: 'center',
                boxShadow: '0 0 0 1px #232b3a',
            }} />
            {/* Start your career on the right */}
            <div style={{
                flex: 1,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'flex-start',
                height: '100%',
            }}>
                <div
                    style={{
                        opacity: canStart ? 1 : 0.5,
                        pointerEvents: canStart ? 'auto' : 'none',
                        background: '#232b3a',
                        border: '2px solid #6366f1',
                        borderRadius: 16,
                        padding: '40px 40px 32px 40px',
                        minWidth: 340,
                        maxWidth: 400,
                        fontSize: '1.1rem',
                        color: canStart ? '#fff' : '#cbd5e1',
                        fontWeight: 400,
                        boxShadow: '0 4px 24px rgba(99,102,241,0.08)',
                        transition: 'opacity 0.3s, color 0.3s',
                        userSelect: 'none',
                        cursor: canStart ? 'pointer' : 'not-allowed',
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                        gap: 24,
                    }}
                >
                    <div style={{ fontWeight: 700, fontSize: '1.5rem', color: canStart ? '#6366f1' : '#888', marginBottom: 8 }}>Start your career</div>
                    <div style={{ textAlign: 'center', color: '#cbd5e1', marginBottom: 8 }}>
                        Ready to take the next step? Begin your journey and see how far you can go!
                    </div>
                    <button
                        disabled={!canStart}
                        style={{
                            background: canStart ? '#6366f1' : '#232b3a',
                            color: canStart ? '#fff' : '#888',
                            border: 'none',
                            borderRadius: 8,
                            padding: '12px 32px',
                            fontSize: '1.1rem',
                            fontWeight: 600,
                            cursor: canStart ? 'pointer' : 'not-allowed',
                            boxShadow: canStart ? '0 2px 8px rgba(99,102,241,0.10)' : 'none',
                            transition: 'all 0.2s',
                            marginTop: 8,
                        }}
                    >
                        Get Started
                    </button>
                </div>
            </div>
        </div>
    );
};

export default Landing;