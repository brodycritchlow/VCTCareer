import React, { useState } from 'react';
import { useState } from 'react';

const rankOptions = [
    'Iron', 'Bronze', 'Silver', 'Gold', 'Platinum', 'Diamond', 'Ascendant', 'Immortal', 'Radiant'
];
const experienceOptions = [
    'Tier 1', 'Tier 2', 'Tier 3', 'None'
];
const divisionOptions = ['1', '2', '3'];

const LandingModal = ({ show, onClose }) => {
    const [age, setAge] = useState('');
    const [rank, setRank] = useState('');
    const [division, setDivision] = useState('');
    const [experience, setExperience] = useState('');
    const [submitted, setSubmitted] = useState(false);
    const [apiResult, setApiResult] = useState(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState(null);
    const [showToast, setShowToast] = useState(false);
    const [toastMsg, setToastMsg] = useState('');

    if (!show) return null;

    const handleToastConfirm = () => {
        if (apiResult) {
            // Save to localStorage
            localStorage.setItem('valorantCareer', JSON.stringify(apiResult));
            // Redirect to /career
            window.location.href = '/career';
        }
        setShowToast(false);
    };

    const handleSubmit = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);
        setApiResult(null);
        try {
            console.log('Submitting career creation:')  
            const res = await fetch('http://127.0.0.1:8080/createCareer', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    age: Number(age),
                    current_rank: rank,
                    past_experience: experience,
                    division: rank === 'Radiant' ? null : division,
                }),
            });
            if (!res.ok) throw new Error('API error');
            const data = await res.json();
            setApiResult(data);
            setSubmitted(true);
            setShowToast(true); // Show toast after successful API
        } catch (err) {
            setError('Failed to create career. Please try again.');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div style={{
            position: 'fixed',
            top: 0,
            left: 0,
            width: '100vw',
            height: '100vh',
            background: 'rgba(26,32,44,0.85)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
        }}>
            {/* Toast notification */}
            {showToast && apiResult && (
                <div style={{
                    position: 'fixed',
                    top: 0,
                    left: 0,
                    width: '100vw',
                    height: '100vh',
                    background: 'rgba(26,32,44,0.7)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    zIndex: 2000,
                }}>
                    <div style={{
                        background: '#232b3a',
                        borderRadius: 16,
                        padding: '32px 28px',
                        minWidth: 280,
                        color: '#fff',
                        boxShadow: '0 8px 32px rgba(0,0,0,0.25)',
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                    }}>
                        <div style={{ fontWeight: 700, fontSize: '1.15rem', marginBottom: 10, color: '#a3e635' }}>
                            Placement Complete!
                        </div>
                        <div style={{ color: '#fff', marginBottom: 12, fontSize: '1.05rem', textAlign: 'center' }}>
                            You have been placed in <span style={{ color: '#6366f1', fontWeight: 600 }}>{apiResult.starting_tier}</span>!
                        </div>
                        <button
                            onClick={handleToastConfirm}
                            style={{
                                background: '#6366f1',
                                color: '#fff',
                                border: 'none',
                                borderRadius: 8,
                                padding: '10px 28px',
                                fontSize: '1rem',
                                fontWeight: 600,
                                cursor: 'pointer',
                                marginTop: 8,
                            }}
                        >
                            OK
                        </button>
                    </div>
                </div>
            )}
            <style>{`
                @keyframes fadeIn {
                    from { opacity: 0; transform: translateY(-16px) translateX(-50%); }
                    to { opacity: 1; transform: translateY(0) translateX(-50%); }
                }
            `}</style>
            <div style={{
                background: '#232b3a',
                borderRadius: 16,
                padding: '40px 32px',
                minWidth: 320,
                maxWidth: 400,
                color: '#fff',
                boxShadow: '0 8px 32px rgba(0,0,0,0.25)',
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                position: 'relative',
            }}>
                <button
                    onClick={onClose}
                    style={{
                        position: 'absolute',
                        top: 16,
                        right: 16,
                        background: 'none',
                        border: 'none',
                        color: '#cbd5e1',
                        fontSize: 24,
                        cursor: 'pointer',
                    }}
                    aria-label="Close modal"
                >
                    ×
                </button>
                <div style={{ fontWeight: 700, fontSize: '1.3rem', marginBottom: 12, textAlign: 'center' }}>
                    Welcome to VCTCareer
                </div>
                <div style={{ textAlign: 'center', color: '#cbd5e1', marginBottom: 20, fontSize: '1rem' }}>
                    A simulator where you get to choose your starting point and develop as a player and a professional.
                </div>
                {submitted && apiResult && !showToast ? (
                    <div style={{ color: '#a3e635', margin: '24px 0', textAlign: 'center' }}>
                        <div style={{ fontWeight: 700, fontSize: '1.1rem', marginBottom: 8 }}>Career Created!</div>
                        <div style={{ color: '#fff', marginBottom: 8 }}>
                            <strong>Starting Tier:</strong> {apiResult.starting_tier}
                        </div>
                        <div style={{ color: '#cbd5e1', fontSize: '0.98rem', marginBottom: 8 }}>
                            <strong>Career Info:</strong>
                            <pre style={{
                                background: '#1a202c',
                                color: '#fff',
                                borderRadius: 8,
                                padding: 12,
                                marginTop: 8,
                                fontSize: '0.95rem',
                                textAlign: 'left',
                                overflowX: 'auto',
                            }}>{JSON.stringify(apiResult.career_info, null, 2)}</pre>
                        </div>
                        <button
                            onClick={onClose}
                            style={{
                                background: '#6366f1',
                                color: '#fff',
                                border: 'none',
                                borderRadius: 8,
                                padding: '12px 32px',
                                fontSize: '1.1rem',
                                fontWeight: 600,
                                cursor: 'pointer',
                                marginTop: 8,
                            }}
                        >
                            Close
                        </button>
                    </div>
                ) : submitted && !apiResult && !loading ? (
                    <div style={{ color: '#f87171', margin: '24px 0', textAlign: 'center' }}>{error || 'Unknown error.'}</div>
                ) : loading ? (
                    <div style={{ color: '#6366f1', margin: '24px 0', textAlign: 'center' }}>Creating your career...</div>
                ) : (
                <form onSubmit={handleSubmit} style={{ width: '100%', display: 'flex', flexDirection: 'column', gap: 18 }}>
                    <label style={{ color: '#cbd5e1', fontWeight: 500, marginBottom: 4 }}>
                        Age
                        <input
                            type="number"
                            min="16"
                            max="99"
                            value={age}
                            onChange={e => setAge(e.target.value)}
                            required
                            style={{
                                width: '100%',
                                marginTop: 4,
                                padding: '8px 12px',
                                borderRadius: 6,
                                border: '1px solid #6366f1',
                                background: '#1a202c',
                                color: '#fff',
                                fontSize: '1rem',
                                outline: 'none',
                                marginBottom: 8,
                            }}
                        />
                    </label>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 8, position: 'relative', minHeight: 70, width: '100%' }}>
                        <label style={{ color: '#cbd5e1', fontWeight: 500, marginBottom: 0, flex: 1, minWidth: 0, maxWidth: '100%' }}>
                            Current Rank
                            <select
                                value={rank}
                                onChange={e => { setRank(e.target.value); setDivision(''); }}
                                required
                                style={{
                                    width: '100%',
                                    minWidth: 0,
                                    maxWidth: '100%',
                                    marginTop: 4,
                                    padding: '8px 12px',
                                    borderRadius: 6,
                                    border: '1px solid #6366f1',
                                    background: '#1a202c',
                                    color: '#fff',
                                    fontSize: '1rem',
                                    outline: 'none',
                                    boxSizing: 'border-box',
                                    overflow: 'hidden',
                                }}
                            >
                                <option value="" disabled>Select rank</option>
                                {rankOptions.map(opt => (
                                    <option key={opt} value={opt}>{opt}</option>
                                ))}
                            </select>
                        </label>
                        <div style={{
                            position: 'relative',
                            width: rank && rank !== 'Radiant' ? 110 : 0,
                            minWidth: 0,
                            maxWidth: 110,
                            marginLeft: rank && rank !== 'Radiant' ? 0 : -110,
                            opacity: rank && rank !== 'Radiant' ? 1 : 0,
                            pointerEvents: rank && rank !== 'Radiant' ? 'auto' : 'none',
                            transition: 'all 0.35s cubic-bezier(.4,2,.6,1)',
                            overflow: 'hidden',
                            display: 'flex',
                            alignItems: 'center',
                        }}>
                        {rank && rank !== 'Radiant' && (
                            <label style={{ color: '#cbd5e1', fontWeight: 500, marginBottom: 0, width: '100%' }}>
                                Division
                                <select
                                    value={division}
                                    onChange={e => setDivision(e.target.value)}
                                    required
                                    style={{
                                        width: '100%',
                                        marginTop: 4,
                                        padding: '8px 12px',
                                        borderRadius: 6,
                                        border: '1px solid #6366f1',
                                        background: '#1a202c',
                                        color: '#fff',
                                        fontSize: '1rem',
                                        outline: 'none',
                                    }}
                                >
                                    <option value="" disabled>Select</option>
                                    {divisionOptions.map(opt => (
                                        <option key={opt} value={opt}>{opt}</option>
                                    ))}
                                </select>
                            </label>
                        )}
                        </div>
                    </div>
                    <label style={{ color: '#cbd5e1', fontWeight: 500, marginBottom: 4 }}>
                        Past Experiences (tier)
                        <select
                            value={rankOptions.indexOf(rank) < 7 && rank ? 'None' : experience}
                            onChange={e => setExperience(e.target.value)}
                            required
                            disabled={rankOptions.indexOf(rank) < 7 && rank}
                            style={{
                                width: '100%',
                                marginTop: 4,
                                padding: '8px 12px',
                                borderRadius: 6,
                                border: '1px solid #6366f1',
                                background: '#1a202c',
                                color: '#fff',
                                fontSize: '1rem',
                                outline: 'none',
                                marginBottom: 8,
                                opacity: rankOptions.indexOf(rank) < 7 && rank ? 0.6 : 1,
                            }}
                        >
                            <option value="" disabled>Select experience</option>
                            {experienceOptions.map(opt => (
                                <option key={opt} value={opt}>{opt}</option>
                            ))}
                        </select>
                    </label>
                    <button
                        type="submit"
                        style={{
                            background: '#6366f1',
                            color: '#fff',
                            border: 'none',
                            borderRadius: 8,
                            padding: '12px 32px',
                            fontSize: '1.1rem',
                            fontWeight: 600,
                            cursor: 'pointer',
                            marginTop: 8,
                        }}
                        disabled={rank && rank !== 'Radiant' && !division}
                    >
                        Submit
                    </button>
                </form>
                )}
                <button
                    onClick={onClose}
                    style={{
                        background: 'none',
                        color: '#cbd5e1',
                        border: 'none',
                        borderRadius: 8,
                        padding: '10px 0 0 0',
                        fontSize: '1rem',
                        fontWeight: 400,
                        cursor: 'pointer',
                        marginTop: 8,
                    }}
                >
                    Close
                </button>
            </div>
        </div>
    );
};

export default LandingModal;
