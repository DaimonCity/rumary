import React, {useEffect, useMemo, useState} from 'react';
import {Navigate, NavLink, useLocation, useNavigate} from 'react-router-dom';
import {
    Activity,
    Boxes,
    Check,
    ChevronRight,
    CircleUserRound,
    Copy,
    Download,
    FileCog,
    FolderOpen,
    Gauge,
    LogOut,
    Menu,
    Moon,
    Plus,
    Play,
    ShieldBan,
    Settings,
    Sun,
    Terminal,
    Trash2,
    Users,
    X
} from 'lucide-react';
import {
    api,
    apiBlob,
    authToken,
    Ban,
    Capabilities,
    Configuration,
    Group,
    GroupSummary,
    Instance,
    json,
    Profile,
    setAuthToken
} from './api';
import {I18nContext, Locale, translate, useI18n} from './i18n';

const INSTANCE_MANAGEMENT = ['instance.create', 'instance.update', 'instance.delete'] as const;
const CONFIGURATION_MANAGEMENT = ['configuration.create', 'configuration.update', 'configuration.delete'] as const;
const GROUP_MANAGEMENT = ['group.create', 'group.delete', 'group.weight.update', 'group.permissions.update', 'group.parents.update', 'group.members.create', 'group.members.delete'] as const;
const MODERATION_MANAGEMENT = ['user.ban', 'user.ban.permanent', 'user.unban'] as const;
const SETTINGS_MANAGEMENT = ['settings.instance_path.update', 'settings.instance_path.delete'] as const;
const ADMINISTRATIVE_PERMISSIONS = [...INSTANCE_MANAGEMENT, ...CONFIGURATION_MANAGEMENT, ...GROUP_MANAGEMENT, ...MODERATION_MANAGEMENT, ...SETTINGS_MANAGEMENT] as const;

const hasPermission = (permissions: ReadonlySet<string>, permission: string) =>
    permissions.has('*') || permissions.has(permission);
const hasAnyPermission = (permissions: ReadonlySet<string>, required: readonly string[]) =>
    required.some(permission => hasPermission(permissions, permission));
const canUseInstancesSection = (permissions: ReadonlySet<string>) =>
    hasPermission(permissions, 'instance.list') && hasAnyPermission(permissions, INSTANCE_MANAGEMENT);
const canUseConfigurationsSection = (permissions: ReadonlySet<string>) =>
    hasPermission(permissions, 'instance.list') &&
    hasPermission(permissions, 'instance.configurations.list') &&
    hasAnyPermission(permissions, CONFIGURATION_MANAGEMENT);
const canManageGroupsSection = (permissions: ReadonlySet<string>) =>
    hasPermission(permissions, 'group.get') && hasAnyPermission(permissions, GROUP_MANAGEMENT);

const navigation = [
    {label: 'overview', to: '/', icon: Gauge, visible: () => true},
    {label: 'instances', to: '/instances', icon: Boxes, visible: canUseInstancesSection},
    {label: 'configurations', to: '/configurations', icon: FileCog, visible: canUseConfigurationsSection},
    {label: 'groupsAndPermissions', to: '/groups', icon: Users, visible: (p: ReadonlySet<string>) => hasPermission(p, 'group.list')},
    {label: 'moderation', to: '/moderation', icon: ShieldBan, visible: (p: ReadonlySet<string>) => hasPermission(p, 'user.ban')},
    {label: 'apiConsole', to: '/api', icon: Terminal, visible: (p: ReadonlySet<string>) => hasAnyPermission(p, ADMINISTRATIVE_PERMISSIONS)},
    {label: 'settings', to: '/settings', icon: Settings, visible: (p: ReadonlySet<string>) => hasAnyPermission(p, SETTINGS_MANAGEMENT)},
    {label: 'account', to: '/account', icon: CircleUserRound, visible: () => true}
] as const;

function ErrorAlert({message, onClose}: { message: string; onClose: () => void }) {
    const {t} = useI18n();
    return <div className="alert error dismissible" role="alert">
        <span>{t(message)}</span>
        <button className="icon-btn alert-close" type="button" onClick={onClose}
                aria-label={t('dismissError')} title={t('dismissError')}><X size={15}/></button>
    </div>
}

function Auth({onLogin}: { onLogin: () => void }) {
    const {t} = useI18n();
    const [mode, setMode] = useState<'login' | 'register'>('login');
    const [form, setForm] = useState({login: '', password: '', nickname: ''});
    const [totp, setTotp] = useState<string | null>(null);
    const [code, setCode] = useState('');
    const [error, setError] = useState('');
    const submit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        try {
            if (totp) {
                const d = await api<{ access_token: string }>('/api/v1/auth/login/totp', json({
                    user_id: totp,
                    totp_code: code
                }));
                setAuthToken(d.access_token);
                onLogin();
                return;
            }
            const d = await api<any>(`/api/v1/auth/${mode}`, json(mode === 'login' ? {
                login: form.login,
                password: form.password
            } : form));
            if (d.user_id) {
                setTotp(d.user_id);
                return;
            }
            setAuthToken(d.access_token);
            onLogin();
        } catch (e) {
            setError((e as Error).message)
        }
    };
    return <main className="auth">
        <div className="auth-card">
            <div className="brand"><span className="brand-mark">R</span>
                <div><strong>Rumary</strong><small>Control plane</small></div>
            </div>
            <h1>{t(totp ? 'confirmSignIn' : mode === 'login' ? 'signInToConsole' : 'createAccount')}</h1><p
            className="muted">{t(totp ? 'enterAuthenticatorCode' : 'manageResourcesDescription')}</p>{error &&
            <ErrorAlert message={error} onClose={() => setError('')}/>}{totp ?
            <form onSubmit={submit}><label>{t('totpCode')}<input autoFocus value={code} onChange={e => setCode(e.target.value)}
                                                          inputMode="numeric" required/></label>
                <button className="primary">{t('confirm')}</button>
            </form> : <form onSubmit={submit}>{mode === 'register' &&
                <label>{t('nickname')}<input value={form.nickname} onChange={e => setForm({...form, nickname: e.target.value})}
                                     required/></label>}<label>{t('login')}<input value={form.login} onChange={e => setForm({
                ...form,
                login: e.target.value
            })} required/></label><label>{t('password')}<input type="password" value={form.password}
                                                      onChange={e => setForm({...form, password: e.target.value})}
                                                      required/></label>
                <button className="primary">{t(mode === 'login' ? 'signIn' : 'signUp')}</button>
            </form>}
            <button className="link" onClick={() => {
                setMode(mode === 'login' ? 'register' : 'login');
                setError('')
            }}>{t(mode === 'login' ? 'noAccountSignUp' : 'haveAccountSignIn')}</button>
        </div>
    </main>
}

function Shell({children, onLogout, permissions}: { children: React.ReactNode; onLogout: () => void; permissions: ReadonlySet<string> }) {
    const [open, setOpen] = useState(false);
    const {locale, setLocale, t} = useI18n();
    const [dark, setDark] = useState(() => localStorage.getItem('rumary_theme') === 'dark');
    useEffect(() => {
        document.documentElement.dataset.theme = dark ? 'dark' : 'light';
        localStorage.setItem('rumary_theme', dark ? 'dark' : 'light');
    }, [dark]);
    return <div className="shell">
        <aside className={open ? 'open' : ''}>
            <div className="side-head">
                <div className="brand"><span className="brand-mark">R</span>
                    <div><strong>Rumary</strong><small>Control plane</small></div>
                </div>
                <button className="icon-btn mobile-only" onClick={() => setOpen(false)}><X size={18}/></button>
            </div>
            <nav>{navigation.filter(item => item.visible(permissions)).map(({label, to, icon: Icon}) => <NavLink key={to} to={to} end={to === '/'}
                                                          onClick={() => setOpen(false)}><Icon
                size={18}/><span>{t(to === '/groups' && !canManageGroupsSection(permissions) ? 'groups' : label)}</span></NavLink>)}</nav>
            <div className="side-foot">
                <div className="status"><i/> {t('apiConnected')}</div>
                <div className="side-preferences"><button className="preference" onClick={() => setLocale(locale === 'ru' ? 'en' : 'ru')}><span>{locale.toUpperCase()}</span></button><button className="preference" onClick={() => setDark(value => !value)} aria-label={dark ? 'Use light theme' : 'Use dark theme'}>{dark ? <Sun size={15}/> : <Moon size={15}/>}</button></div>
                <button className="logout" onClick={onLogout}><LogOut size={16}/> {t('signOut')}</button>
            </div>
        </aside>
        <section className="workspace">
            <header className="topbar">
                <button className="icon-btn mobile-only" onClick={() => setOpen(true)}><Menu size={20}/></button>
                <div className="crumb">{t('controlConsole')} <ChevronRight size={14}/> <span>Rumary</span></div>
                <div className="top-actions"><CircleUserRound size={18}/><span>{t('account')}</span></div>
            </header>
            <div className="content">{children}</div>
        </section>
    </div>
}

const Card = ({title, children, action}: { title: string; children: React.ReactNode; action?: React.ReactNode }) => {
    const {t} = useI18n();
    return <section className="panel">
        <div className="panel-head"><h2>{t(title)}</h2>{action}</div>
        {children}</section>;
};

function Dashboard({profile, permissions}: { profile: Profile; permissions: ReadonlySet<string> }) {
    const {t} = useI18n();
    const showInstances = canUseInstancesSection(permissions);
    const showConfigurations = canUseConfigurationsSection(permissions);
    const [instances, setInstances] = useState<Instance[]>([]);
    const [configs, setConfigs] = useState<Configuration[]>([]);
    const [health, setHealth] = useState<'ok' | 'error' | 'loading'>('loading');
    useEffect(() => {
        api<{ status: string }>('/health').then(() => setHealth('ok')).catch(() => setHealth('error'));
        if (!showInstances && !showConfigurations) return;
        api<Instance[]>('/api/v1/instances').then(xs => {
            setInstances(xs);
            return Promise.all(xs.map(i => api<Configuration[]>(`/api/v1/instance/${i.id}/configurations`)))
        }).then(all => setConfigs(all.flat())).catch(() => {
        });
    }, [showConfigurations, showInstances]);
    return <>
        <div className="page-title">
            <div><span className="eyebrow">{t('workspace')}</span><h1>{t('overview')}</h1><p className="muted">{t('welcome')}, {profile.nickname || profile.login}</p></div>
            <div className="avatar">{(profile.nickname || profile.login).slice(0, 1).toUpperCase()}</div>
        </div>
        <div className="stats">
            {showInstances && <div><span>{t('availableInstances')}</span><b>{instances.length}</b><small>{t('syncedWithApi')}</small></div>}
            {showConfigurations && <div><span>{t('configurations')}</span><b>{configs.length}</b><small>{t('linkedToAvailableInstances')}</small></div>}
            <div><span>API</span><b
                className={health === 'ok' ? 'good' : health === 'error' ? 'danger' : ''}>{health === 'ok' ? t('operational') : health === 'error' ? t('unavailable') : t('checking')}</b><small>{t('check')} `/health`</small></div>
            <div><span>2FA</span><b
                className={profile.has_totp ? 'good' : ''}>{profile.has_totp ? t('enabled') : t('notConfigured')}</b><small>{t('accountProtection')}</small></div>
        </div>
        <div className="grid-2">{showInstances && <Card title="recentInstances"
                                      action={<NavLink className="text-btn" to="/instances">{t('all')} <ChevronRight
                                          size={14}/></NavLink>}>{instances.length ?
            <div className="list">{instances.slice(0, 4).map(i => <NavLink to={`/instances/${i.id}`}
                                                                           className="list-row" key={i.id}><span
                className="resource-icon">{i.icon || '◈'}</span><span><strong>{i.display_name}</strong><small>{i.version} · {i.loader}</small></span><ChevronRight
                size={16}/></NavLink>)}</div> : <Empty text="noInstancesYet"/>}</Card>}<Card
            title="systemStatus">
            <div className="system-state">
                <div className="state-icon"><Activity size={22}/></div>
                <div>
                    <strong>{health === 'error' ? t('apiUnavailable') : health === 'loading' ? t('checkingApi') : t('allServicesOperational')}</strong>
                    <p className="muted">{t('lastCheckedJustNow')}</p></div>
                <span
                    className={`pill ${health === 'ok' ? 'green' : health === 'error' ? 'red' : 'gray'}`}>{health === 'ok' ? t('ok') : health === 'error' ? t('error') : '…'}</span>
            </div>
            <div className="meter"><span style={{width: '86%'}}/></div>
            <small className="muted">API latency · 86 ms</small></Card></div>
    </>
}

function InstanceDetail({permissions}: { permissions: ReadonlySet<string> }) {
    const {t} = useI18n();
    const id = useLocation().pathname.split('/')[2];
    const [instance, setInstance] = useState<Instance | null>(null);
    const [configs, setConfigs] = useState<Configuration[]>([]);
    const [error, setError] = useState('');
    const [edit, setEdit] = useState(false);
    useEffect(() => {
        if (!id) return;
        api<Instance>(`/api/v1/instance/${id}`).then(setInstance).then(() => api<Configuration[]>(`/api/v1/instance/${id}/configurations`)).then(setConfigs).catch(e => setError(e.message));
    }, [id]);
    if (error && !instance) return <ErrorAlert message={error} onClose={() => setError('')}/>;
    if (!instance) return <div className="loading">{t('loadingInstance')}</div>;
    const update = async (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        const f = new FormData(e.currentTarget);
        try {
            const updated = await api<Instance>(`/api/v1/instance/${id}`, json({
                icon: f.get('icon'), dir_name: f.get('dir_name'), display_name: f.get('display_name'),
                version: f.get('version'), description: f.get('description'), loader: f.get('loader'),
                loader_version: f.get('loader_version') || null
            }, 'PATCH'));
            setInstance(updated);
            setEdit(false);
        } catch (e) {
            setError((e as Error).message)
        }
    };
    const remove = async () => {
        if (!window.confirm(t('deleteInstanceConfirm'))) return;
        try {
            await api(`/api/v1/instance/${id}`, {method: 'DELETE'});
            window.location.assign('/instances');
        } catch (e) {
            setError((e as Error).message)
        }
    };
    return <>{error && <ErrorAlert message={error} onClose={() => setError('')}/>}<Title title={instance.display_name}
                    subtitle={`${instance.version} · ${instance.loader}${instance.loader_version ? ` ${instance.loader_version}` : ''}`}
                    action={<div className="modal-actions"><NavLink className="secondary"
                                                                    to="/instances">{t('back')}</NavLink>
                        {hasPermission(permissions, 'instance.update') && <button className="secondary" onClick={() => setEdit(true)}>{t('edit')}</button>}
                        {hasPermission(permissions, 'instance.delete') && <button className="secondary danger" onClick={remove}><Trash2 size={15}/>{t('delete')}</button>}
                    </div>}/><Card
        title="description"><p className="muted detail-copy">{instance.description || t('noDescriptionProvided')}</p>
        <div className="detail-stats">
            <div><span>{t('directory')}</span><code>{instance.dir_name}</code></div>
            <div><span>{t('loader')}</span><b>{instance.loader}</b></div>
            <div><span>{t('configurations')}</span><b>{configs.length}</b></div>
        </div>
    </Card><Card title="instanceConfigurations">{configs.length ?
        <div className="resource-grid">{configs.map(c => <div className="resource-card" key={c.id}>
            <div className="resource-icon large">{c.icon || '▦'}</div>
            <div><h3>{c.display_name}</h3><p className="muted">{Object.keys(c.files || {}).length} {t('files')}</p>
                <code>{c.dir_name}</code></div>
        </div>)}</div> : <Empty text="noConfigurationsYet"/>}</Card>{edit && <Modal title="editInstance" onClose={() => setEdit(false)}><form onSubmit={update} className="form-grid"><label>{t('name')}<input name="display_name" defaultValue={instance.display_name} required/></label><label>{t('directory')}<input name="dir_name" defaultValue={instance.dir_name} required/></label><label>{t('version')}<input name="version" defaultValue={instance.version} required/></label><label>{t('loader')}<input name="loader" defaultValue={instance.loader} required/></label><label>{t('loaderVersion')}<input name="loader_version" defaultValue={instance.loader_version || ''}/></label><label>{t('icon')}<input name="icon" defaultValue={instance.icon}/></label><label className="full">{t('description')}<textarea name="description" defaultValue={instance.description} rows={3}/></label><div className="modal-actions full"><button type="button" className="secondary" onClick={() => setEdit(false)}>{t('cancel')}</button><button className="primary">{t('save')}</button></div></form></Modal>}</>
}

function Empty({text}: { text: string }) {
    const {t} = useI18n();
    return <div className="empty"><FolderOpen size={28}/><span>{t(text)}</span></div>
}

function Instances({permissions}: { permissions: ReadonlySet<string> }) {
    const {t} = useI18n();
    const [items, setItems] = useState<Instance[]>([]);
    const [show, setShow] = useState(false);
    const [error, setError] = useState('');
    const load = () => api<Instance[]>('/api/v1/instances').then(setItems).catch(e => setError(e.message));
    useEffect(() => {
        void load()
    }, []);
    const create = async (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        const f = new FormData(e.currentTarget);
        try {
            await api('/api/v1/instance', json({
                icon: f.get('icon'),
                dir_name: f.get('dir_name'),
                display_name: f.get('display_name'),
                version: f.get('version'),
                description: f.get('description'),
                loader: f.get('loader'),
                loader_version: f.get('loader_version') || null,
                is_public: f.get('is_public') === 'on',
                share_with: []
            }));
            setShow(false);
            await load();
        } catch (e) {
            setError((e as Error).message)
        }
    };
    return <><Title title="instances" subtitle="gameVersionsLoadersAccess"
                    action={hasPermission(permissions, 'instance.create') ? <button className="primary small" onClick={() => setShow(true)}><Plus size={16}/> {t('newInstance')}</button> : undefined}/>{error && <ErrorAlert message={error} onClose={() => setError('')}/>}<Card
        title={`${items.length} ${t('resources')}`}>
        <div className="table-wrap">
            <table>
                <thead>
                <tr>
                    <th>{t('instance')}</th>
                    <th>{t('version')}</th>
                    <th>{t('loader')}</th>
                    <th>{t('directory')}</th>
                    <th/>
                </tr>
                </thead>
                <tbody>{items.map(i => <tr key={i.id}>
                    <td><span className="table-name"><span
                        className="resource-icon">{i.icon || '◈'}</span><span><strong>{i.display_name}</strong><small>{i.description || t('noDescription')}</small></span></span>
                    </td>
                    <td>{i.version}</td>
                    <td>{i.loader}{i.loader_version && ` ${i.loader_version}`}</td>
                    <td><code>{i.dir_name}</code></td>
                    <td><NavLink className="icon-btn" to={`/instances/${i.id}`}><ChevronRight size={16}/></NavLink></td>
                </tr>)}</tbody>
            </table>
            {!items.length && <Empty text="createFirstInstance"/>}</div>
    </Card>{show && <Modal title="newInstance" onClose={() => setShow(false)}>
        <form onSubmit={create} className="form-grid"><label>{t('name')}<input name="display_name"
                                                                            required/></label><label>{t('directory')}<input
            name="dir_name" required/></label><label>{t('version')}<input name="version" placeholder="1.21.1" required/></label><label>{t('loader')}<input
            name="loader" placeholder="fabric" required/></label><label>{t('loaderVersion')}<input name="loader_version"/></label><label>{t('icon')}<input
            name="icon" placeholder="◈"/></label><label className="full">{t('description')}<textarea name="description" rows={3}/></label><label
            className="check full"><input type="checkbox" name="is_public"/> {t('publicResource')}</label>
            <div className="modal-actions full">
                <button type="button" className="secondary" onClick={() => setShow(false)}>{t('cancel')}</button>
                <button className="primary">{t('create')}</button>
            </div>
        </form>
    </Modal>}</>
}

function Configurations({permissions}: { permissions: ReadonlySet<string> }) {
    const {t} = useI18n();
    const [items, setItems] = useState<Configuration[]>([]);
    const [instances, setInstances] = useState<Instance[]>([]);
    const [show, setShow] = useState(false);
    const [editing, setEditing] = useState<Configuration | null>(null);
    const [error, setError] = useState('');
    const load = () => api<Instance[]>('/api/v1/instances').then(xs => {
        setInstances(xs);
        return Promise.all(xs.map(i => api<Configuration[]>(`/api/v1/instance/${i.id}/configurations`)))
    }).then(all => setItems(all.flat())).catch(e => setError(e.message));
    useEffect(() => {
        void load()
    }, []);
    const save = async (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        const f = new FormData(e.currentTarget);
        try {
            if (editing) await api(`/api/v1/configuration/${editing.id}`, json({
                icon: f.get('icon'),
                dir_name: f.get('dir_name'),
                display_name: f.get('display_name'),
                instance_id: f.get('instance_id')
            }, 'PATCH'));
            else await api('/api/v1/configuration', json({
                icon: f.get('icon'),
                dir_name: f.get('dir_name'),
                display_name: f.get('display_name'),
                instance_id: f.get('instance_id'),
                is_public: f.get('is_public') === 'on',
                share_with: []
            }));
            setShow(false);
            setEditing(null);
            await load();
        } catch (e) {
            setError((e as Error).message)
        }
    };
    const remove = async (id: string) => {
        if (!window.confirm(t('deleteConfigurationConfirm'))) return;
        try {
            await api(`/api/v1/configuration/${id}`, {method: 'DELETE'});
            await load()
        } catch (e) {
            setError((e as Error).message)
        }
    };
    const download = async (configId: string, filepath: string) => {
        try {
            const path = filepath.split('/').map(encodeURIComponent).join('/');
            const blob = await apiBlob(`/api/v1/download/${configId}/${path}`);
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filepath.split('/').pop() || 'file';
            a.click();
            URL.revokeObjectURL(url)
        } catch (e) {
            setError((e as Error).message)
        }
    };
    return <><Title title="configurations" subtitle="buildsAndFileSets"
                    action={hasPermission(permissions, 'configuration.create') ? <button className="primary small" onClick={() => {
                        setEditing(null);
                        setShow(true)
                    }}><Plus size={16}/> {t('newConfiguration')}</button> : undefined}/>{error &&
        <ErrorAlert message={error} onClose={() => setError('')}/>}<Card title={`${items.length} ${t('configurationsCount')}`}>
        <div className="resource-grid">{items.map(c => <div className="resource-card" key={c.id}>
            <div className="resource-icon large">{c.icon || '▦'}</div>
            <div className="resource-card-body"><h3>{c.display_name}</h3><p
                className="muted">{instances.find(i => i.id === c.instance_id)?.display_name || t('instance')} · {c.files ? Object.keys(c.files).length : 0} {t('files')}</p>
                <code>{c.dir_name}</code>
                <div className="resource-actions">
                    {hasPermission(permissions, 'configuration.update') && <button className="text-btn" onClick={async () => {
                        try {
                            const detail = await api<Configuration>(`/api/v1/configuration/${c.id}`);
                            setEditing(detail);
                            setShow(true)
                        } catch (e) {
                            setError((e as Error).message)
                        }
                    }}>{t('open')}
                    </button>}
                    {hasPermission(permissions, 'configuration.delete') && <button className="text-btn danger" onClick={() => remove(c.id)}><Trash2 size={13}/>{t('delete')}</button>}
                </div>
                {hasPermission(permissions, 'configuration.download') && Object.keys(c.files || {}).slice(0, 4).map(file => <button className="file-link" key={file}
                                                                            onClick={() => download(c.id, file)}>
                    <Download size={13}/>{file}</button>)}</div>
        </div>)}{!items.length && <Empty text="noConfigurationsYet"/>}</div>
    </Card>{show && <Modal title={editing ? 'editConfiguration' : 'newConfiguration'} onClose={() => {
        setShow(false);
        setEditing(null)
    }}>
        <form onSubmit={save} className="form-grid"><label>{t('name')}<input name="display_name"
                                                                          defaultValue={editing?.display_name || ''}
                                                                          required/></label><label>{t('directory')}<input
            name="dir_name" defaultValue={editing?.dir_name || ''} required/></label><label>{t('icon')}<input name="icon"
                                                                                                         defaultValue={editing?.icon || '▦'}/></label><label>{t('instance')}<select
            name="instance_id" defaultValue={editing?.instance_id || instances[0]?.id || ''}
            required>{instances.map(i => <option key={i.id}
                                                 value={i.id}>{i.display_name}</option>)}</select></label>{!editing &&
            <label className="check full"><input type="checkbox" name="is_public"/> {t('publicResource')}</label>}
            <div className="modal-actions full">
                <button type="button" className="secondary" onClick={() => {
                    setShow(false);
                    setEditing(null)
                }}>{t('cancel')}
                </button>
                <button className="primary">{t('save')}</button>
            </div>
        </form>
    </Modal>}</>
}

function Groups({permissions}: { permissions: ReadonlySet<string> }) {
    const {t} = useI18n();
    const canManage = canManageGroupsSection(permissions);
    const canCreate = hasPermission(permissions, 'group.create');
    const canDelete = hasPermission(permissions, 'group.delete');
    const canChangeWeight = hasPermission(permissions, 'group.weight.update');
    const canChangePermissions = hasPermission(permissions, 'group.permissions.update');
    const canChangeParents = hasPermission(permissions, 'group.parents.update');
    const canAddMembers = hasPermission(permissions, 'group.members.create');
    const canRemoveMembers = hasPermission(permissions, 'group.members.delete');
    const [groups, setGroups] = useState<GroupSummary[]>([]);
    const [selected, setSelected] = useState<Group | null>(null);
    const [show, setShow] = useState(false);
    const [error, setError] = useState('');
    const load = () => api<GroupSummary[]>('/api/v1/groups?limit=100').then(setGroups).catch(e => setError(e.message));
    const select = (name: string) => api<Group>(`/api/v1/groups/${encodeURIComponent(name)}`).then(setSelected).catch(e => setError(e.message));
    useEffect(() => {
        void load()
    }, []);
    const create = async (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        const f = new FormData(e.currentTarget);
        try {
            await api('/api/v1/groups', json({name: f.get('name'), weight: Number(f.get('weight'))}));
            setShow(false);
            await load();
        } catch (e) {
            setError((e as Error).message)
        }
    };
    const groupPath = selected ? `/api/v1/groups/${encodeURIComponent(selected.name)}` : '';
    const mutate = async (request: Promise<unknown>) => {
        try {
            await request;
            if (selected) await select(selected.name);
            await load()
        } catch (e) {
            setError((e as Error).message)
        }
    };
    const removeGroup = async () => {
        if (!selected || !window.confirm(t('deleteGroupConfirm'))) return;
        try {
            await api(`/api/v1/groups/${encodeURIComponent(selected.name)}`, {method: 'DELETE'});
            setSelected(null);
            await load()
        } catch (e) {
            setError((e as Error).message)
        }
    };
    if (!canManage) return <><Title title="groups" subtitle="availableUserGroups"/>
        {error && <ErrorAlert message={error} onClose={() => setError('')}/>}<Card title="groups">
            <div className="list">{groups.map(g => <div className="list-row" key={g.name}>
                <span className="group-dot"/><span><strong>{g.name}</strong><small>{t('weight')} {g.weight}</small></span>
            </div>)}{!groups.length && <Empty text="noGroupsYet"/>}</div>
        </Card></>;

    return <><Title title="groupsAndPermissions" subtitle="rolesPrioritiesPermissions"
                    action={canCreate ? <button className="primary small" onClick={() => setShow(true)}><Plus size={16}/> {t('newGroup')}</button> : undefined}/>{error && <ErrorAlert message={error} onClose={() => setError('')}/>}
        <div className="split"><Card title="groups">
            <div className="list">{groups.map(g => <button
                className={`list-row selectable ${selected?.name === g.name ? 'selected' : ''}`} key={g.name}
                onClick={() => select(g.name)}>
                <span
                    className="group-dot"/><span><strong>{g.name}</strong><small>{t('weight')} {g.weight}</small></span><ChevronRight
                size={16}/></button>)}{!groups.length && <Empty text="noGroupsYet"/>}</div>
        </Card><Card title={selected ? selected.name : 'selectGroup'}>{selected ? <>
            <div className="detail-stats">
                <div><span>{t('weight')}</span><b>{selected.weight}</b></div>
                <div><span>{t('members')}</span><b>{selected.members.length}</b></div>
                <div><span>{t('parents')}</span><b>{selected.parents.length}</b></div>
            </div>
            {(canChangeWeight || canDelete) && <div className="group-toolbar">
                {canChangeWeight && <form className="inline-form compact" onSubmit={e => {
                    e.preventDefault();
                    const f = new FormData(e.currentTarget);
                    void mutate(api(groupPath + '/weight', json({weight: Number(f.get('weight'))}, 'PUT')))
                }}><input name="weight" type="number" defaultValue={selected.weight} required/>
                    <button className="secondary">{t('weight')}</button>
                </form>}
                {canDelete && <button className="secondary danger" onClick={removeGroup}><Trash2 size={14}/>{t('deleteGroup')}</button>}
            </div>}
            <h3 className="subhead">{t('permissions')}</h3>
            <div className="permission-list">{selected.permissions.map(p => <div key={p.key}><code>{p.key}</code><span
                className={`pill ${p.allow ? 'green' : 'red'}`}>{p.allow ? t('allowed') : t('denied')}
                {canChangePermissions && <button className="icon-btn" title={t('revoke')}
                        onClick={() => void mutate(api(groupPath + '/permissions', json({
                            grant: [],
                            revoke: [p.key]
                        }, 'PATCH')))}><X size={12}/></button>}</span>
            </div>)}{!selected.permissions.length && <p className="muted">{t('noPermissionsSet')}</p>}
                {canChangePermissions && <form className="inline-form compact" onSubmit={e => {
                    e.preventDefault();
                    const f = new FormData(e.currentTarget);
                    void mutate(api(groupPath + '/permissions', json({
                        grant: [{
                            key: f.get('key'),
                            allow: f.get('allow') === 'allow'
                        }], revoke: []
                    }, 'PATCH')))
                }}><input name="key" placeholder="configuration.get" required/><select name="allow">
                    <option value="allow">{t('allow')}</option>
                    <option value="deny">{t('deny')}</option>
                </select>
                    <button className="secondary">{t('apply')}</button>
                </form>}
            </div>
            <h3 className="subhead">{t('inheritance')}</h3>
            <div className="permission-list">{selected.parents.map(parent => <div key={parent}><code>{parent}</code>
                {canChangeParents && <button className="text-btn danger"
                        onClick={() => void mutate(api(`${groupPath}/parents/${encodeURIComponent(parent)}`, {method: 'DELETE'}))}>{t('delete')}
                </button>}
            </div>)}
                {canChangeParents && <form className="inline-form compact" onSubmit={e => {
                    e.preventDefault();
                    const f = new FormData(e.currentTarget);
                    void mutate(api(groupPath + '/parents', json({parent: f.get('parent')})))
                }}><input name="parent" placeholder="parent-group" required/>
                    <button className="secondary">{t('addParent')}</button>
                </form>}
            </div>
            <h3 className="subhead">{t('members')}</h3>
            <div className="permission-list">{selected.members.map(member => <div key={member}><code>{member}</code>
                {canRemoveMembers && <button className="text-btn danger"
                        onClick={() => void mutate(api(`${groupPath}/members/${member}`, {method: 'DELETE'}))}>{t('delete')}
                </button>}
            </div>)}
                {canAddMembers && <form className="inline-form compact" onSubmit={e => {
                    e.preventDefault();
                    const f = new FormData(e.currentTarget);
                    void mutate(api(groupPath + '/members', json({
                        user_id: f.get('user_id'),
                        expires_at: f.get('expires_at') ? new Date(String(f.get('expires_at'))).toISOString() : null
                    })))
                }}><input name="user_id" placeholder={t('userUuid')} required/><input name="expires_at"
                                                                                          type="datetime-local"/>
                    <button className="secondary">{t('addMember')}</button>
                </form>}
            </div>
        </> : <Empty text="selectGroupOnLeft"/>}</Card></div>
        {show && canCreate && <Modal title="newGroup" onClose={() => setShow(false)}>
            <form onSubmit={create}><label>{t('groupName')}<input name="name" placeholder="moderator"
                                                            required/></label><label>{t('weight')}<input name="weight"
                                                                                               type="number" min="0"
                                                                                               defaultValue="10"
                                                                                               required/></label>
                <div className="modal-actions">
                    <button type="button" className="secondary" onClick={() => setShow(false)}>{t('cancel')}</button>
                    <button className="primary">{t('create')}</button>
                </div>
            </form>
        </Modal>}</>
}

function Moderation({permissions}: { permissions: ReadonlySet<string> }) {
    const {t, locale} = useI18n();
    const [userId, setUserId] = useState('');
    const [user, setUser] = useState<Profile | null>(null);
    const [bans, setBans] = useState<Ban[]>([]);
    const [show, setShow] = useState(false);
    const [error, setError] = useState('');
    const load = () => {
        if (!userId) return;
        setError('');
        Promise.all([api<Profile>(`/api/v1/user/${userId}`), api<Ban[]>(`/api/v1/user/${userId}/bans`)]).then(([profile, history]) => {
            setUser(profile);
            setBans(history)
        }).catch(e => setError(e.message))
    };
    return <><Title title="moderation" subtitle="userBansAndHistory"/><Card
        title="userProfile">
        <div className="inline-form"><input value={userId} onChange={e => setUserId(e.target.value)}
                                            placeholder={t('userUuid')}/>
            <button className="secondary" onClick={load}>{t('load')}</button>
            {userId && hasPermission(permissions, 'user.ban') &&
                <button className="primary" onClick={() => setShow(true)}><ShieldBan size={16}/> {t('issueBan')}</button>}
        </div>
        {user && <div className="detail-copy"><strong>{user.nickname || user.login}</strong><p
            className="muted">{user.login} · 2FA {user.has_totp ? t('enabledLower') : t('notConfiguredLower')}</p></div>}
    </Card>{bans.length > 0 && <Card title="banHistory">
        <div className="table-wrap">
            <table>
                <thead>
                <tr>
                    <th>{t('scope')}</th>
                    <th>{t('reason')}</th>
                    <th>{t('start')}</th>
                    <th>{t('end')}</th>
                    <th>{t('status')}</th>
                    <th/>
                </tr>
                </thead>
                <tbody>{bans.map(b => <tr key={b.id}>
                    <td>{b.scope}</td>
                    <td>{b.reason_code}</td>
                    <td>{new Date(b.starts_at).toLocaleString(locale === 'ru' ? 'ru-RU' : 'en-US')}</td>
                    <td>{b.expires_at ? new Date(b.expires_at).toLocaleString(locale === 'ru' ? 'ru-RU' : 'en-US') : t('never')}</td>
                    <td><span
                        className={`pill ${b.revoked_at ? 'gray' : 'red'}`}>{b.revoked_at ? t('revoked') : t('active')}</span>
                    </td>
                    <td>{!b.revoked_at && hasPermission(permissions, 'user.unban') && <button className="text-btn danger" onClick={async () => {
                        try {
                            await api(`/api/v1/user/${userId}/bans/${b.id}`, {
                                method: 'DELETE',
                                body: JSON.stringify({reason: t('revokedFromConsole')})
                            });
                            load();
                        } catch (e) {
                            setError((e as Error).message)
                        }
                    }}>{t('revoke')}</button>}</td>
                </tr>)}</tbody>
            </table>
        </div>
    </Card>}{error && <ErrorAlert message={error} onClose={() => setError('')}/>}{show && hasPermission(permissions, 'user.ban') &&
        <Modal title="issueBan" onClose={() => setShow(false)}>
            <form onSubmit={async e => {
                e.preventDefault();
                const f = new FormData(e.currentTarget);
                try {
                    await api(`/api/v1/user/${userId}/bans`, json({
                        scope: f.get('scope'),
                        reason_code: f.get('reason_code'),
                        staff_note: f.get('staff_note') || null,
                        starts_at: null,
                        expires_at: f.get('expires_at') ? new Date(String(f.get('expires_at'))).toISOString() : null
                    }));
                    setShow(false);
                    load();
                } catch (e) {
                    setError((e as Error).message)
                }
            }}><label>{t('scope')}<select name="scope">
                <option value="account">{t('account')}</option>
                <option value="api">API</option>
                <option value="launcher">{t('launcher')}</option>
                <option value="game">{t('game')}</option>
            </select></label><label>{t('reasonCode')}<input name="reason_code" required/></label><label>{t('end')}<input
                name="expires_at" type="datetime-local" required={!hasPermission(permissions, 'user.ban.permanent')}/></label><label>{t('note')}<textarea name="staff_note"
                                                                                         rows={3}/></label>
                <div className="modal-actions">
                    <button type="button" className="secondary" onClick={() => setShow(false)}>{t('cancel')}</button>
                    <button className="primary">{t('block')}</button>
                </div>
            </form>
        </Modal>}</>
}

function SettingsPage({permissions}: { permissions: ReadonlySet<string> }) {
    const {t} = useI18n();
    const [path, setPath] = useState('');
    const [saved, setSaved] = useState(false);
    const [error, setError] = useState('');
    return <><Title title="settings" subtitle="storageAndSessionSettings"/><Card title="dataPath">
        <form className="inline-form" onSubmit={async e => {
            e.preventDefault();
            try {
                await api('/api/v1/settings/instance_path', json({path}));
                setSaved(true)
            } catch (e) {
                setError((e as Error).message)
            }
        }}><input value={path} onChange={e => setPath(e.target.value)} placeholder="/var/lib/rumary/instances"
                  required/>
            {hasPermission(permissions, 'settings.instance_path.update') && <button className="primary">{t('save')}</button>}
            {hasPermission(permissions, 'settings.instance_path.delete') && <button type="button" className="secondary danger" onClick={async () => {
                try {
                    await api('/api/v1/settings/instance_path', {method: 'DELETE'});
                    setPath('');
                    setSaved(true)
                } catch (e) {
                    setError((e as Error).message)
                }
            }}>{t('reset')}
            </button>}
        </form>
        {saved && <div className="alert success">{t('pathSaved')}</div>}{error &&
        <ErrorAlert message={error} onClose={() => setError('')}/>}</Card></>
}

function Account({profile, onDeleted}: { profile: Profile; onDeleted: () => void }) {
    const {t} = useI18n();
    const [error, setError] = useState('');
    const [password, setPassword] = useState('');
    const remove = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!window.confirm(t('deleteAccountConfirm'))) return;
        try {
            await api('/api/v1/users/me', json({password}, 'DELETE'));
            setAuthToken(null);
            onDeleted()
        } catch (e) {
            setError((e as Error).message)
        }
    };
    return <><Title title="account" subtitle="profileAndSessionManagement"/><Card title="profile">
        <div className="detail-stats">
            <div><span>{t('login')}</span><b>{profile.login}</b></div>
            <div><span>{t('nickname')}</span><b>{profile.nickname}</b></div>
            <div><span>2FA</span><b>{profile.has_totp ? t('enabled') : t('notConfigured')}</b></div>
        </div>
    </Card><Card title="deleteAccount"><p className="detail-copy muted">{t('deleteAccountWarning')}</p>
        <form className="inline-form" onSubmit={remove}><input type="password" value={password}
                                                               onChange={e => setPassword(e.target.value)}
                                                               placeholder={t('currentPassword')} required/>
            <button className="secondary danger"><Trash2 size={15}/>{t('deleteAccount')}</button>
        </form>
        {error && <ErrorAlert message={error} onClose={() => setError('')}/>}</Card></>
}

type ApiRoute = { method: string; path: string; note: string };
const apiRoutes: ApiRoute[] = [
    {method: 'GET', path: '/health', note: 'health check'},
    {method: 'POST', path: '/api/v1/auth/register', note: 'register'},
    {method: 'POST', path: '/api/v1/auth/login', note: 'login'},
    {method: 'POST', path: '/api/v1/auth/login/totp', note: 'verify TOTP'},
    {method: 'POST', path: '/api/v1/auth/refresh', note: 'refresh session'},
    {method: 'POST', path: '/api/v1/auth/logout', note: 'logout'},
    {method: 'GET', path: '/api/v1/users/me', note: 'current profile'},
    {method: 'DELETE', path: '/api/v1/users/me', note: 'delete account'},
    {method: 'GET', path: '/api/v1/user/{user_id}', note: 'user profile'},
    {method: 'GET', path: '/api/v1/user/{user_id}/bans', note: 'list bans'},
    {method: 'POST', path: '/api/v1/user/{user_id}/bans', note: 'create ban'},
    {method: 'DELETE', path: '/api/v1/user/{user_id}/bans/{ban_id}', note: 'revoke ban'},
    {method: 'POST', path: '/api/v1/settings/instance_path', note: 'set instances path'},
    {method: 'DELETE', path: '/api/v1/settings/instance_path', note: 'remove instances path'},
    {method: 'GET', path: '/api/v1/instances', note: 'list instances'},
    {method: 'POST', path: '/api/v1/instance', note: 'create instance'},
    {method: 'GET', path: '/api/v1/instance/{instance_id}', note: 'get instance'},
    {method: 'PATCH', path: '/api/v1/instance/{instance_id}', note: 'update instance'},
    {method: 'DELETE', path: '/api/v1/instance/{instance_id}', note: 'delete instance'},
    {method: 'GET', path: '/api/v1/instance/{instance_id}/configurations', note: 'list configurations'},
    {method: 'POST', path: '/api/v1/configuration', note: 'create configuration'},
    {method: 'GET', path: '/api/v1/configuration/{config_id}', note: 'get configuration'},
    {method: 'PATCH', path: '/api/v1/configuration/{config_id}', note: 'update configuration'},
    {method: 'DELETE', path: '/api/v1/configuration/{config_id}', note: 'delete configuration'},
    {method: 'GET', path: '/api/v1/download/{config_id}/{filepath}', note: 'download file'},
    {method: 'GET', path: '/api/v1/groups?limit=100', note: 'list groups'},
    {method: 'POST', path: '/api/v1/groups', note: 'create group'},
    {method: 'GET', path: '/api/v1/groups/{name}', note: 'get group'},
    {method: 'DELETE', path: '/api/v1/groups/{name}', note: 'delete group'},
    {method: 'PUT', path: '/api/v1/groups/{name}/weight', note: 'update group weight'},
    {method: 'PATCH', path: '/api/v1/groups/{name}/permissions', note: 'update permissions'},
    {method: 'POST', path: '/api/v1/groups/{name}/parents', note: 'add parent'},
    {method: 'DELETE', path: '/api/v1/groups/{name}/parents/{parent}', note: 'remove parent'},
    {method: 'POST', path: '/api/v1/groups/{name}/members', note: 'add member'},
    {method: 'DELETE', path: '/api/v1/groups/{name}/members/{user_id}', note: 'remove member'}
];

function ApiExplorer() {
    const {t} = useI18n();
    const [selected, setSelected] = useState(0);
    const [method, setMethod] = useState(apiRoutes[0].method);
    const [path, setPath] = useState(apiRoutes[0].path);
    const [body, setBody] = useState('');
    const [result, setResult] = useState('');
    const [error, setError] = useState('');
    const choose = (index: number) => {
        const route = apiRoutes[index];
        setSelected(index);
        setMethod(route.method);
        setPath(route.path);
        setBody('');
        setResult('');
        setError('');
    };
    const run = async (event: React.FormEvent) => {
        event.preventDefault();
        setError('');
        setResult('');
        try {
            let init: RequestInit = {method};
            if (body.trim()) {
                const parsed = JSON.parse(body);
                init = json(parsed, method);
            }
            const response = await api<unknown>(path, init);
            setResult(response === undefined ? '204 No Content' : JSON.stringify(response, null, 2));
        } catch (e) {
            setError((e as Error).message);
        }
    };
    return <><Title title="apiConsole" subtitle="callAllApiRoutes"/>
        <div className="api-console">
            <Card title="apiRoutes"><div className="route-list">{apiRoutes.map((route, index) => <button key={`${route.method}-${route.path}`} className={`route-row ${index === selected ? 'selected' : ''}`} onClick={() => choose(index)}><span className={`method method-${route.method.toLowerCase()}`}>{route.method}</span><code>{route.path}</code><small>{route.note}</small></button>)}</div></Card>
            <Card title="newRequest"><form onSubmit={run} className="request-form"><div className="request-line"><label>{t('method')}<select value={method} onChange={e => setMethod(e.target.value)}>{['GET', 'POST', 'PATCH', 'PUT', 'DELETE'].map(value => <option key={value}>{value}</option>)}</select></label><label className="path-field">{t('path')}<input value={path} onChange={e => setPath(e.target.value)} required/></label><button className="primary" type="submit"><Play size={15}/>{t('runRequest')}</button></div><label>{t('requestBodyJson')}<textarea value={body} onChange={e => setBody(e.target.value)} rows={8} placeholder={'{\n  "key": "value"\n}'}/></label></form></Card>
            {(error || result) && <Card title="apiResponse">{error ? <ErrorAlert message={error} onClose={() => setError('')}/> : <><div className="result-toolbar"><button className="text-btn" onClick={() => navigator.clipboard?.writeText(result)}><Copy size={14}/>{t('copy')}</button><span className="pill green"><Check size={12}/> {t('requestCompleted')}</span></div><pre className="api-result">{result}</pre></>}</Card>}
        </div>
    </>;
}

function Title({title, subtitle, action}: { title: string; subtitle: string; action?: React.ReactNode }) {
    const {t} = useI18n();
    return <div className="page-title">
        <div><span className="eyebrow">{t('management')}</span><h1>{t(title)}</h1><p className="muted">{t(subtitle)}</p></div>
        {action}</div>
}

function Modal({title, onClose, children}: { title: string; onClose: () => void; children: React.ReactNode }) {
    const {t} = useI18n();
    return <div className="backdrop" onMouseDown={e => e.target === e.currentTarget && onClose()}>
        <div className="modal">
            <div className="modal-head"><h2>{t(title)}</h2>
                <button className="icon-btn" onClick={onClose}><X size={18}/></button>
            </div>
            {children}
        </div>
    </div>
}

export default function App() {
    const [profile, setProfile] = useState<Profile | null>(null);
    const [permissions, setPermissions] = useState<Set<string> | null>(null);
    const [loading, setLoading] = useState(!!authToken());
    const [locale, setLocaleState] = useState<Locale>(() => (localStorage.getItem('rumary_locale') as Locale) || 'ru');
    const nav = useNavigate();
    const location = useLocation();
    const setLocale = (next: Locale) => {
        setLocaleState(next);
        localStorage.setItem('rumary_locale', next);
        document.documentElement.lang = next;
    };
    const i18n = useMemo(() => ({locale, setLocale, t: (value: string) => translate(locale, value)}), [locale]);
    useEffect(() => {
        document.documentElement.lang = locale;
    }, [locale]);
    const loadSession = async () => {
        const [nextProfile, capabilities] = await Promise.all([
            api<Profile>('/api/v1/users/me'),
            api<Capabilities>('/api/v1/users/me/capabilities')
        ]);
        setProfile(nextProfile);
        setPermissions(new Set(capabilities.permissions));
    };
    useEffect(() => {
        if (authToken()) void loadSession().catch(() => {
            setAuthToken(null);
            setProfile(null);
            setPermissions(null);
        }).finally(() => setLoading(false)); else setLoading(false)
    }, []);
    if (loading) return <I18nContext.Provider value={i18n}><div className="loading">{translate(locale, 'loadingConsole')}</div></I18nContext.Provider>;
    if (!profile || !permissions) return <I18nContext.Provider value={i18n}><Auth onLogin={() => void loadSession()}/></I18nContext.Provider>;
    const path = location.pathname;
    const canManageInstances = canUseInstancesSection(permissions);
    const canManageConfigurations = canUseConfigurationsSection(permissions);
    const canModerate = hasPermission(permissions, 'user.ban');
    const canManageSettings = hasAnyPermission(permissions, SETTINGS_MANAGEMENT);
    const canUseApiConsole = hasAnyPermission(permissions, ADMINISTRATIVE_PERMISSIONS);
    const guard = (allowed: boolean, page: React.ReactNode) => allowed ? page : <Navigate to="/" replace/>;
    let page: React.ReactNode = path.startsWith('/instances/') ? guard(canManageInstances, <InstanceDetail permissions={permissions}/>) : path === '/instances' ?
        guard(canManageInstances, <Instances permissions={permissions}/>) : path === '/configurations' ? guard(canManageConfigurations, <Configurations permissions={permissions}/>) : path === '/groups' ?
            guard(hasPermission(permissions, 'group.list'), <Groups permissions={permissions}/>) : path === '/moderation' ? guard(canModerate, <Moderation permissions={permissions}/>) : path === '/settings' ?
                guard(canManageSettings, <SettingsPage permissions={permissions}/>) : path === '/account' ? <Account profile={profile} onDeleted={() => {
                        setProfile(null);
                        setPermissions(null);
                        nav('/')
                    }}/> :
        path === '/api' ? guard(canUseApiConsole, <ApiExplorer/>) : <Dashboard profile={profile} permissions={permissions}/>;
    return <I18nContext.Provider value={i18n}><Shell permissions={permissions} onLogout={async () => {
        try {
            await api('/api/v1/auth/logout', {method: 'POST'})
        } catch {
        }
        setAuthToken(null);
        setProfile(null);
        setPermissions(null);
        nav('/')
    }}>{page}</Shell></I18nContext.Provider>
}
